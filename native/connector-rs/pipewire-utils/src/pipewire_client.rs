use std::{
    cell::{OnceCell, RefCell},
    collections::{hash_map::Entry, HashMap, HashSet},
    rc::Rc,
    str::FromStr,
};

use libspa::utils::dict::DictRef;
use pipewire::{
    context::ContextRc,
    core::CoreRc,
    keys,
    link::Link,
    main_loop::MainLoopRc,
    node::{Node, NodeInfoRef},
    properties::properties,
    proxy::ProxyT,
    registry::{GlobalObject, Listener, RegistryRc},
    types::ObjectType,
};
use thiserror::Error;
use tracing::instrument;

use crate::{
    cancellation_signal::CancellationSignal,
    proxies::ProxyRefs,
    roundtrip::{Scheduler, StopSettings, StopSettingsBuilder},
};

#[derive(Error, Debug)]
#[error("internal pipewire error {0:?}")]
pub struct PipewireError(#[from] pipewire::Error);

pub type Result<T, E = PipewireError> = std::result::Result<T, E>;

#[derive(Clone)]
pub struct PipewireClient {
    mainloop: MainLoopRc,
    #[allow(unused)]
    context: ContextRc,
    core: CoreRc,
}

pub enum IterationAction {
    Stop,
    ScheduleRoundtrip,
}

#[derive(Clone)]
struct IterationContext {
    registry: RegistryRc,
    scheduler: Scheduler,
}

#[derive(Debug)]
pub struct NodeProperties {
    pub application_name: Option<String>,
    pub media_name: String,
    pub object_serial: i64,

    pub media_class: String,
}

#[derive(Debug)]
pub struct OutputNode {
    pub id: u32,
    pub properties: NodeProperties,
}

#[derive(Error, Debug)]
pub enum OutputNodeCreationError {
    #[error("missing property dictionary")]
    MissingPropertyDict,
    #[error("missing some properties: {properties:?}")]
    MissingProperties { properties: Vec<&'static str> },
}

impl TryFrom<&NodeInfoRef> for OutputNode {
    type Error = OutputNodeCreationError;
    fn try_from(value: &NodeInfoRef) -> Result<Self, Self::Error> {
        let Some(props) = value.props() else {
            return Err(OutputNodeCreationError::MissingPropertyDict);
        };
        let application_name = props.get("application.name");
        let object_serial = props.get("object.serial");
        let media_class = props.get("media.class");
        let media_name = props.get("media.name");
        let (Some(object_serial), Some(media_class), Some(media_name)) =
            (object_serial, media_class, media_name)
        else {
            return Err(OutputNodeCreationError::MissingProperties {
                properties: [
                    (object_serial, "object.serial"),
                    (media_class, "media.class"),
                    (media_name, "media.name"),
                ]
                .into_iter()
                .flat_map(|(prop, name)| prop.is_none().then_some(name))
                .collect(),
            });
        };

        Ok(OutputNode {
            id: value.id(),
            properties: {
                NodeProperties {
                    application_name: application_name.map(|s| s.to_owned()),
                    media_name: media_name.to_owned(),
                    object_serial: object_serial.parse().unwrap(),
                    media_class: media_class.to_owned(),
                }
            },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    INPUT,
    OUTPUT,
}

#[derive(Debug)]
pub struct InvalidPortError;
impl FromStr for PortDirection {
    type Err = InvalidPortError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "in" => Ok(PortDirection::INPUT),
            "out" => Ok(PortDirection::OUTPUT),
            _ => Err(InvalidPortError),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioChannel<'a> {
    FrontLeft,
    FrontRight,
    Other(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StereoAudioChannel {
    FrontLeft,
    FrontRight,
}

impl<'a> From<&'a str> for AudioChannel<'a> {
    fn from(value: &'a str) -> Self {
        match value {
            "FL" => AudioChannel::FrontLeft,
            "FR" => AudioChannel::FrontRight,
            s => AudioChannel::Other(s),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ports {
    pub fl_port: u32,
    pub fr_port: u32,
}

impl Ports {
    fn get_stereo_channel<'a>(&self, channel: &StereoAudioChannel) -> u32 {
        match channel {
            &StereoAudioChannel::FrontLeft => self.fl_port,
            &StereoAudioChannel::FrontRight => self.fr_port,
        }
    }
    fn get_channel<'a>(&self, channel: &AudioChannel<'a>) -> Option<&u32> {
        match channel {
            &AudioChannel::FrontLeft => Some(&self.fl_port),
            &AudioChannel::FrontRight => Some(&self.fr_port),
            &AudioChannel::Other(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NodeWithPorts {
    pub id: u32,
    pub ports: Ports,
}

#[derive(Debug, Clone, Copy)]
pub struct MaybePorts {
    pub fl_port: Option<u32>,
    pub fr_port: Option<u32>,
}

impl MaybePorts {
    pub fn both(self) -> Option<Ports> {
        let MaybePorts { fl_port, fr_port } = self;
        let fl_port = fl_port?;
        let fr_port = fr_port?;
        Some(Ports { fl_port, fr_port })
    }
}

#[derive(Debug, Clone, Copy)]
struct PortInfo<'a> {
    channel: AudioChannel<'a>,
    node_id: u32,
    id: u32,
    direction: PortDirection,
}

#[derive(Debug)]
struct OwnedPortInfo {
    channel: Option<StereoAudioChannel>,
    node_id: u32,
    id: u32,
    direction: PortDirection,
}

#[derive(Error, Debug)]
#[error("port channel is not stereo")]
pub struct ChannelIsNotStereoError;
impl<'a> TryFrom<AudioChannel<'a>> for StereoAudioChannel {
    type Error = ChannelIsNotStereoError;
    fn try_from(value: AudioChannel<'a>) -> Result<Self, Self::Error> {
        match value {
            AudioChannel::FrontLeft => Ok(StereoAudioChannel::FrontLeft),
            AudioChannel::FrontRight => Ok(StereoAudioChannel::FrontRight),
            AudioChannel::Other(_) => Err(ChannelIsNotStereoError),
        }
    }
}

impl<'a> From<PortInfo<'a>> for OwnedPortInfo {
    fn from(
        PortInfo {
            channel,
            node_id,
            id,
            direction,
        }: PortInfo<'a>,
    ) -> Self {
        OwnedPortInfo {
            id,
            node_id,
            channel: channel.try_into().ok(),
            direction,
        }
    }
}

impl IterationContext {
    #[instrument(skip_all)]
    fn stop(&self) {
        tracing::trace!("stopping scheduler");
        self.scheduler.stop();
    }

    #[instrument(skip_all)]
    fn schedule_roundtrip(&self) {
        tracing::trace!("scheduling roundtrip");
        self.scheduler.schedule_roundtrip();
    }
}

struct NodeListenerHandle {
    #[allow(unused)]
    registry_listener: Listener,
    #[allow(unused)]
    proxies: Rc<RefCell<ProxyRefs>>,
}

impl PipewireClient {
    pub fn new() -> Result<Self> {
        let mainloop = MainLoopRc::new(None)?;
        let context = ContextRc::new(&mainloop, None)?;
        let core = context.connect_rc(None)?;

        Ok(Self {
            mainloop,
            context,
            core,
        })
    }

    fn create_scheduler(&self) -> Scheduler {
        //let mainloop = MainLoopRc::new(None).unwrap();
        //let context = ContextRc::new(&mainloop, None).unwrap();
        //let core = context.connect_rc(None).unwrap();

        //Scheduler::new(mainloop, core)
        Scheduler::new(self.mainloop.clone(), self.core.clone())
    }

    fn do_roundtrip(&self) {
        self.create_scheduler().run(
            StopSettingsBuilder::default()
                .stop_on_last_roundtrip()
                .build(),
        );
    }

    fn iterate_globals(
        &self,
        stop_settings: StopSettings,
        global_callback: impl Fn(&IterationContext, &GlobalObject<&DictRef>) + 'static,
    ) {
        let scheduler = self.create_scheduler();

        let registry = self.core.get_registry_rc().unwrap();

        let iteration_context = IterationContext {
            registry: registry.clone(),
            scheduler: scheduler.clone(),
        };
        let reg_listener = registry
            .add_listener_local()
            .global(move |global| global_callback(&iteration_context, global))
            .register();

        scheduler.run(stop_settings);

        drop(reg_listener);
    }

    fn add_node_listener<F, DF>(
        &self,
        registry: RegistryRc,
        node_callback: F,
        node_destroy_callback: DF,
        scheduler: Option<Scheduler>,
    ) -> NodeListenerHandle
    where
        F: Fn(&NodeInfoRef) + Clone + 'static,
        DF: Fn(u32) + 'static,
    {
        let proxies: Rc<RefCell<ProxyRefs>> = Rc::new(RefCell::new(ProxyRefs::new()));

        let listener = registry
            .add_listener_local()
            .global({
                let proxies = proxies.clone();
                let registry = registry.clone();
                move |global| {
                    if !matches!(global.type_, ObjectType::Node) {
                        return;
                    }
                    tracing::trace!(global_id = global.id, "global");
                    let Ok(proxy) = registry.bind::<Node, _>(global) else {
                        tracing::debug!("global {} is not assignable to node", global.id);
                        return;
                    };
                    let listener = proxy
                        .add_listener_local()
                        .info({
                            let node_callback = node_callback.clone();
                            move |node| {
                                node_callback(node);
                            }
                        })
                        .register();
                    if let Some(scheduler) = scheduler.as_ref() {
                        scheduler.schedule_roundtrip();
                    }
                    proxies
                        .borrow_mut()
                        .add_proxy(Box::new(proxy), vec![Box::new(listener)]);
                }
            })
            .global_remove({
                let proxies = proxies.clone();
                move |global_id| {
                    if proxies.borrow_mut().remove_proxy(&global_id) {
                        node_destroy_callback(global_id);
                    }
                }
            })
            .register();

        NodeListenerHandle {
            registry_listener: listener,
            proxies,
        }
    }

    fn iterate_nodes<F>(&self, stop_settings: StopSettings, object_callback: F)
    where
        F: Fn(&IterationContext, &NodeInfoRef) + Clone + 'static,
    {
        let scheduler = self.create_scheduler();

        let registry = self.core.get_registry_rc().unwrap();

        let iteration_context = IterationContext {
            registry: registry.clone(),
            scheduler: scheduler.clone(),
        };
        let reg_listener = self.add_node_listener(
            registry,
            move |node| object_callback(&iteration_context, node),
            |_| {},
            Some(scheduler.clone()),
        );
        scheduler.run(stop_settings);

        drop(reg_listener);
    }

    fn extract_port_info<'a>(global: &'a GlobalObject<&'a DictRef>) -> Option<PortInfo<'a>> {
        let props = global.props?;

        if global.type_ != ObjectType::Port {
            return None;
        }

        let id: u32 = global.id;
        let node_id = props.get(*keys::NODE_ID)?.parse().unwrap();
        let direction = props.get(*keys::PORT_DIRECTION)?.parse().unwrap();
        let channel = AudioChannel::from(props.get(*keys::AUDIO_CHANNEL)?);
        Some(PortInfo {
            channel,
            id,
            node_id,
            direction,
        })
    }

    fn is_global_a_port_for_node<'a>(
        global: &'a GlobalObject<&'a DictRef>,
        direction: PortDirection,
        node_id: u32,
    ) -> Option<PortInfo<'a>> {
        let port_info = Self::extract_port_info(global)?;

        (port_info.node_id == node_id && port_info.direction == direction).then_some(port_info)
    }

    fn find_fl_fr_ports(
        &self,
        node_id: u32,
        direction: PortDirection,
        only_existent: bool,
    ) -> MaybePorts {
        let fl_port = Rc::new(OnceCell::new());
        let fr_port = Rc::new(OnceCell::new());

        self.iterate_globals(
            StopSettingsBuilder::default()
                .set_stop_on_last_roundtrip(only_existent)
                .build(),
            {
                let fl_port = fl_port.clone();
                let fr_port = fr_port.clone();
                move |iteration_context, global| {
                    let Some(port) = Self::is_global_a_port_for_node(global, direction, node_id)
                    else {
                        return;
                    };

                    tracing::debug!("found port {:?} for node {}", port, node_id);
                    // Save port id into channel cell
                    if let Some(channel_cell) = match port.channel {
                        AudioChannel::FrontLeft => Some(&fl_port),
                        AudioChannel::FrontRight => Some(&fr_port),
                        AudioChannel::Other(_) => None,
                    } {
                        channel_cell
                            .set(port.id)
                            .expect("Node has multiple ports for same channel");
                    }

                    // Stop searching if we found all ports
                    let found_all_ports = [&fl_port, &fr_port]
                        .iter()
                        .all(|port_id| port_id.get().is_some());

                    tracing::trace!(found_all_ports);

                    if found_all_ports {
                        iteration_context.stop();
                    }
                }
            },
        );

        let fl_port = Rc::into_inner(fl_port).unwrap().into_inner();
        let fr_port = Rc::into_inner(fr_port).unwrap().into_inner();

        MaybePorts { fl_port, fr_port }
    }

    fn await_find_fl_fr_ports(&self, node_id: u32, direction: PortDirection) -> Ports {
        self.find_fl_fr_ports(node_id, direction, false)
            .both()
            .unwrap()
    }

    pub fn await_create_node(&self, node_name: impl AsRef<str>) -> Result<NodeWithPorts> {
        let node = self.core.create_object::<Node>(
            "adapter",
            &properties! {
                *keys::FACTORY_NAME => "support.null-audio-sink",
                *keys::NODE_NAME => node_name.as_ref(),
                *keys::MEDIA_CLASS => "Audio/Source/Virtual",
                "audio.position" => "[FL, FR]",
                *keys::OBJECT_LINGER => "1",
            },
        )?;

        Ok(self.await_node_creation(node))
    }

    pub fn destroy_global(&self, global_id: u32) -> Result<()> {
        let registry = self.core.get_registry()?;
        registry.destroy_global(global_id);
        self.do_roundtrip();
        Ok(())
    }

    fn unlink_ports(&self, ports: HashSet<u32>) {
        tracing::debug!("removing links to: {:?}", &ports);
        let is_link_to_port = move |global: &GlobalObject<&DictRef>| {
            if global.type_ != ObjectType::Link {
                return false;
            }
            let Some(props) = global.props else {
                return false;
            };
            let Some(output_port) = props.get("link.input.port") else {
                return false;
            };
            let Ok(output_port) = output_port.parse::<u32>() else {
                return false;
            };
            ports.contains(&output_port)
        };

        self.iterate_globals(
            StopSettingsBuilder::default()
                .stop_on_last_roundtrip()
                .build(),
            {
                move |iteration_context, global| {
                    if is_link_to_port(global) {
                        tracing::debug!("removing link: {}", global.id);
                        iteration_context.registry.destroy_global(global.id);
                        iteration_context.schedule_roundtrip();
                    }
                }
            },
        );
        tracing::debug!("links removed");
    }

    pub fn unlink_node_ports(&self, ports: Ports) {
        self.unlink_ports(HashSet::from_iter([ports.fl_port, ports.fr_port]));
    }

    fn await_node_creation(&self, node: Node) -> NodeWithPorts {
        let node_id = Rc::new(OnceCell::new());

        tracing::trace!("awaiting to find node");
        let listener = node
            .upcast()
            .add_listener_local()
            .bound({
                let node_id = node_id.clone();
                move |id| {
                    tracing::trace!("node {id} was bound");
                    node_id.set(id).unwrap()
                }
            })
            .register();

        self.do_roundtrip();
        drop(listener);

        let node_id = Rc::into_inner(node_id).unwrap().into_inner().unwrap();

        tracing::trace!("awaiting to find ports");
        let ports = self.await_find_fl_fr_ports(node_id, PortDirection::INPUT);

        NodeWithPorts { id: node_id, ports }
    }

    fn link_ports(&self, from: u32, to: u32) -> Result<Link, pipewire::Error> {
        self.core.create_object::<Link>(
            "link-factory",
            &properties! {
                *keys::LINK_INPUT_PORT => to.to_string(),
                *keys::LINK_OUTPUT_PORT => from.to_string(),
                *keys::OBJECT_LINGER => "1",
            },
        )
    }

    #[instrument(skip(self))]
    fn try_link_ports(&self, from: u32, to: u32) {
        match self.link_ports(from, to) {
            Err(err) => tracing::warn!("failed to link ports: {err}"),
            Ok(link) => tracing::debug!(link_id = link.upcast().id(), "linked ports"),
        };
    }

    pub fn link_nodes(&self, from_id: u32, to_ports: Ports) {
        let from_ports = self.find_fl_fr_ports(from_id, PortDirection::OUTPUT, true);

        let ports = [from_ports.fl_port, from_ports.fr_port]
            .into_iter()
            .zip([to_ports.fl_port, to_ports.fr_port])
            .zip([AudioChannel::FrontLeft, AudioChannel::FrontRight]);

        for ((from, to), channel) in ports {
            let Some(from) = from else {
                tracing::warn!("missing output port for channel {channel:?} in node {from_id}");
                continue;
            };
            self.try_link_ports(from, to);
        }
		self.do_roundtrip();
    }

    pub fn monitor_and_connect_nodes(
        &self,
        target_ports: Ports,
        cancellation_signal: CancellationSignal,
    ) -> Result<()> {
        let scheduler = self.create_scheduler();
        let registry = self.core.get_registry_rc()?;

        let tracked_nodes = Rc::new(RefCell::new(HashSet::new()));
        let nodes_to_ports = Rc::new(RefCell::new(HashMap::<u32, HashSet<u32>>::new()));
        let ports = Rc::new(RefCell::new(HashMap::<u32, OwnedPortInfo>::new()));

        let node_listener = self.add_node_listener(
            registry.clone(),
            {
                let tracked_nodes = tracked_nodes.clone();
                let nodes_to_ports = nodes_to_ports.clone();
                let ports = ports.clone();
                let client = self.clone();
                move |node| {
                    let Some(props) = node.props() else {
                        return;
                    };
                    // TODO: Add exclusions
                    if props.get(*keys::MEDIA_CLASS) != Some("Stream/Output/Audio") {
                        return;
                    }

                    let node_id: u32 = node.id();

                    tracing::debug!("node {node_id} created");
                    tracked_nodes.borrow_mut().insert(node_id);

                    if let Some(port_ids) = nodes_to_ports.borrow().get(&node_id) {
                        tracing::trace!(port_ids = format!("{port_ids:?}"));
                        let ports = ports.borrow();
                        for port_id in port_ids {
                            let port_info = ports.get(port_id).expect("there must be info");

                            let Some(channel) = port_info.channel else {
                                return;
                            };
                            let target_port = target_ports.get_stereo_channel(&channel);
                            client.try_link_ports(*port_id, target_port);
                        }
                    }
                }
            },
            {
                let tracked_nodes = tracked_nodes.clone();
                move |node_id| {
                    tracing::debug!("node {node_id} destroyed");
                    tracked_nodes.borrow_mut().remove(&node_id);
                }
            },
            None,
        );

        let port_listener = registry
            .add_listener_local()
            .global({
                let tracked_nodes = tracked_nodes.clone();
                let nodes_to_ports = nodes_to_ports.clone();
                let ports = ports.clone();
                let client = self.clone();
                move |global_object| {
                    let Some(port_info) = Self::extract_port_info(global_object) else {
                        return;
                    };

                    if port_info.direction != PortDirection::OUTPUT {
                        return;
                    }

                    match nodes_to_ports.borrow_mut().entry(port_info.node_id) {
                        Entry::Vacant(entry) => {
                            entry.insert(HashSet::from_iter([port_info.id]));
                        }
                        Entry::Occupied(mut entry) => {
                            let node_ports = entry.get_mut();
                            node_ports.insert(port_info.id);
                        }
                    }

                    ports
                        .borrow_mut()
                        .insert(port_info.id, OwnedPortInfo::from(port_info));

                    tracing::trace!("port {} created", port_info.id);
                    if tracked_nodes.borrow().contains(&port_info.node_id) {
                        tracing::debug!("port {} is from a tracked node", port_info.id);
                        let Some(target_port) = target_ports.get_channel(&port_info.channel) else {
                            tracing::debug!("unmapped channel: {:?}", port_info.channel);
                            return;
                        };
                        client.try_link_ports(port_info.id, *target_port);
                    }
                }
            })
            .global_remove({
                let nodes_to_ports = nodes_to_ports.clone();
                let ports = ports.clone();
                move |global_id| {
                    // remove port from node
                    {
                        let ports = ports.borrow();
                        let Some(port_info) = ports.get(&global_id) else {
                            // global was not registered as a port
                            return;
                        };

                        nodes_to_ports
                            .borrow_mut()
                            .get_mut(&port_info.node_id)
                            .unwrap()
                            .remove(&global_id);
                        drop(ports);
                    }
                    // remove port
                    ports.borrow_mut().remove(&global_id);
                }
            })
            .register();

        scheduler.run(cancellation_signal.into());

        drop(node_listener);
        drop(port_listener);

        Ok(())
    }

    pub fn find_node_by_name(&self, node_name: impl AsRef<str> + 'static) -> Option<u32> {
        let result = Rc::new(OnceCell::new());
        self.iterate_nodes(
            StopSettingsBuilder::default()
                .stop_on_last_roundtrip()
                .build(),
            {
                let result = result.clone();
                let node_name = Rc::new(node_name);
                move |iteration_context, node| {
                    if node
                        .props()
                        .and_then(|props| props.get(*pipewire::keys::NODE_NAME))
                        .is_some_and(|name| (*node_name).as_ref() == name)
                    {
                        result.set(node.id()).unwrap();
                        iteration_context.stop();
                    }
                }
            },
        );
        Rc::try_unwrap(result).unwrap().into_inner()
    }

    pub fn list_output_nodes(&self) -> Vec<OutputNode> {
        let output_nodes = Rc::new(RefCell::new(vec![]));
        tracing::debug!("iterating over nodes");
        self.iterate_nodes(
            StopSettingsBuilder::default()
                .stop_on_last_roundtrip()
                .build(),
            {
                let output_nodes = output_nodes.clone();
                move |_iteration_context, node| match OutputNode::try_from(node) {
                    Ok(node) => {
                        if node.properties.media_class == "Stream/Output/Audio" {
                            tracing::trace!(node = format_args!("{node:?}"), "adding node");
                            output_nodes.borrow_mut().push(node);
                        } else {
                            tracing::debug!("node {} is not an output", node.id);
                        }
                    }
                    Err(err) => {
                        tracing::trace!(
                            "node {} does not contain all required properties: {}",
                            node.id(),
                            err
                        )
                    }
                }
            },
        );
        Rc::into_inner(output_nodes).unwrap().into_inner()
    }

    pub fn get_node_id_from_object_serial(&self, serial: i64) -> Option<u32> {
        let result = Rc::new(OnceCell::new());
        self.iterate_nodes(
            StopSettingsBuilder::default()
                .stop_on_last_roundtrip()
                .build(),
            {
                let result = result.clone();
                move |iteration_context, node| {
                    let Ok(node) = OutputNode::try_from(node) else {
                        return;
                    };
                    if node.properties.object_serial == serial {
                        result.set(node.id).unwrap();
                        iteration_context.stop();
                    }
                }
            },
        );
        Rc::try_unwrap(result).unwrap().into_inner()
    }
}
