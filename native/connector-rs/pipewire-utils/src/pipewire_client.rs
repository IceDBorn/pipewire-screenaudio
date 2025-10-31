use std::{
    cell::{OnceCell, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
};

use libspa::utils::dict::DictRef;
use pipewire::{
    context::ContextRc,
    core::CoreRc,
    keys,
    link::Link,
    main_loop::MainLoopRc,
    node::{Node, NodeChangeMask, NodeInfoRef},
    properties::properties,
    proxy::ProxyT,
    registry::{GlobalObject, Listener, RegistryRc},
    types::ObjectType,
};
use thiserror::Error;
use tracing::instrument;

use crate::{
    cancellation_signal::CancellationSignal,
    monitor::{LinkTrackerHandle, NodeAndPortsRegistry},
    pipewire_objects::*,
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
                    let Ok(proxy) = registry.bind::<Node, _>(global) else {
                        tracing::trace!("global {} is not assignable to node", global.id);
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

                    tracing::debug!(port_id = port.id, node_id, "found port for node");
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
        tracing::debug!(?ports, "removing links to ports");
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
                        tracing::trace!(link_id = global.id, "removing link");
                        iteration_context.registry.destroy_global(global.id);
                        iteration_context.schedule_roundtrip();
                    }
                }
            },
        );
        tracing::trace!("links removed");
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
        let ports = self.await_find_fl_fr_ports(node_id, PortDirection::Input);

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
            Ok(_) => tracing::trace!("linked ports"),
        };
    }

    pub fn link_nodes(&self, from_id: u32, to_ports: Ports) {
        let from_ports = self.find_fl_fr_ports(from_id, PortDirection::Output, true);

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
        media_name_filter: impl Fn(Option<&str>) -> bool + Clone + 'static,
    ) -> Result<()> {
        let scheduler = self.create_scheduler();
        let registry = self.core.get_registry_rc()?;

        let links_by_port = Rc::new(RefCell::new(HashMap::new()));

        let mut node_and_ports_registry = NodeAndPortsRegistry::default();
        node_and_ports_registry.set_relevant_port_added_callback({
            let client = self.clone();
            let links_by_port = links_by_port.clone();
            move |port_info| {
                let Some(channel) = port_info.channel else {
                    tracing::debug!(
                        port = port_info.id,
                        channel = ?port_info.channel,
                        "unmapped channel"
                    );
                    return;
                };
                let target_port = target_ports.get_stereo_channel(&channel);
                match client.link_ports(port_info.id, target_port) {
                    Ok(link) => {
                        if let Some(prev_handle) = links_by_port
                            .borrow_mut()
                            .insert(port_info.id, LinkTrackerHandle::new(link))
                        {
                            tracing::warn!(
                                prev_id = prev_handle.id(),
                                "linked twice without unlinking"
                            );
                        }
                        tracing::debug!(source_port = port_info.id, target_port, "linked ports");
                    }
                    Err(err) => {
                        tracing::warn!(?err, "error while trying to link ports");
                    }
                }
            }
        });
        node_and_ports_registry.set_relevant_port_removed_callback({
            let registry = registry.clone();
            let links_by_port = links_by_port.clone();
            move |port_info| {
                let port_id = port_info.id;
                tracing::debug!(port_id, "port is no longer relevant, unlinking");
                let Some(link_handle) = links_by_port.borrow_mut().remove(&port_id) else {
                    tracing::warn!(
                        port_id,
                        "trying to unlink port, but link is not being tracked"
                    );
                    return;
                };
                let Some(link_id) = link_handle.id() else {
                    tracing::warn!(port_id, "link does not have a defined id");
                    return;
                };
                registry.destroy_global(link_id);
            }
        });

        let node_listener = self.add_node_listener(
            registry.clone(),
            {
                let node_and_ports_registry = node_and_ports_registry.clone();
                move |node| {
                    let node_id: u32 = node.id();

                    if !node.change_mask().contains(NodeChangeMask::PROPS) {
                        return;
                    }

                    let is_node_relevant = node.props().is_some_and(|props| {
                        props.get(*keys::MEDIA_CLASS) == Some("Stream/Output/Audio")
                            && media_name_filter(props.get(*keys::MEDIA_NAME))
                    });

                    tracing::trace!(node_id, is_node_relevant, "node props updated");
                    let was_node_relevant = node_and_ports_registry.is_node_relevant(node_id);
                    if was_node_relevant != is_node_relevant {
                        if is_node_relevant {
                            node_and_ports_registry.add_relevant_node(node_id);
                        } else {
                            node_and_ports_registry.try_remove_relevant_node(node_id);
                        }
                    } else {
                        tracing::trace!(node_id, "node relevancy has not changed");
                    }
                }
            },
            {
                let node_and_ports_registry = node_and_ports_registry.clone();
                move |node_id| {
                    tracing::trace!(node_id, "node removed");
                    node_and_ports_registry.try_remove_relevant_node(node_id);
                }
            },
            None,
        );

        let port_listener = registry
            .add_listener_local()
            .global({
                let node_and_ports_registry = node_and_ports_registry.clone();
                move |global_object| {
                    let Some(port_info) = Self::extract_port_info(global_object) else {
                        return;
                    };

                    if port_info.direction != PortDirection::Output {
                        return;
                    }

                    tracing::trace!(port_id = port_info.id, "port created");
                    node_and_ports_registry.add_port(port_info);
                }
            })
            .global_remove({
                let node_and_ports_registry = node_and_ports_registry.clone();
                move |global_id| {
                    if node_and_ports_registry.try_remove_port(global_id).is_some() {
                        tracing::trace!(port_id = global_id, "port removed");
                    }
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
        tracing::trace!("iterating over nodes");
        self.iterate_nodes(
            StopSettingsBuilder::default()
                .stop_on_last_roundtrip()
                .build(),
            {
                let output_nodes = output_nodes.clone();
                move |_iteration_context, node| match OutputNode::try_from(node) {
                    Ok(node) => {
                        if node.properties.media_class == "Stream/Output/Audio" {
                            tracing::trace!(?node, "adding node");
                            output_nodes.borrow_mut().push(node);
                        } else {
                            tracing::trace!(node_id = node.id, "node is not an output");
                        }
                    }
                    Err(err) => {
                        tracing::trace!(
                            node_id = node.id(),
                            ?err,
                            "node does not contain all required properties",
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
