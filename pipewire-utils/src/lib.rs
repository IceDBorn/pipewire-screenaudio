use std::{
    any::Any,
    cell::{Cell, OnceCell, RefCell},
    ops::Deref,
    rc::Rc,
};

mod proxies;
mod roundtrip;

use libspa::{pod::Pod, utils::result::AsyncSeq};
use pipewire::{
    core::{Core, CoreRc, PW_ID_CORE},
    keys,
    link::Link,
    main_loop::MainLoopRc,
    node::{Node, NodeInfoRef, NodeListener},
    properties::properties,
    proxy::{Listener, ProxyT},
    registry::{GlobalObject, Registry, RegistryRc},
    spa::utils::dict::DictRef,
    types::ObjectType,
};

use crate::{proxies::ProxyRefs, roundtrip::Scheduler};

pub fn iterate_nodes<F>(
    mainloop: &MainLoopRc,
    core: &CoreRc,
    registry: &RegistryRc,
    object_callback: F,
    only_existent: bool,
) where
    F: Fn(&NodeInfoRef) -> bool + Clone + 'static,
{
    let scheduler = Scheduler::new(mainloop.clone(), core.clone());
    let mut proxies: Rc<RefCell<ProxyRefs>> = Rc::new(RefCell::new(Default::default()));

    let reg_listener = registry
        .add_listener_local()
        .global({
            let mainloop = mainloop.clone();
            let core = core.clone();
            let registry = registry.clone();
            let scheduler = scheduler.clone();
            let proxies = proxies.clone();
            move |global| {
                if !matches!(global.type_, ObjectType::Node) {
                    return;
                }
                let Ok(proxy) = registry.bind::<Node, _>(global) else {
                    log::debug!("global {} is not assignable to node", global.id);
                    return;
                };
                let listener = proxy
                    .add_listener_local()
                    .info({
                        let mainloop = mainloop.clone();
                        let registry = registry.clone();
                        let object_callback = object_callback.clone();
                        move |node| {
                            if object_callback(node) {
                                mainloop.quit();
                            }
                        }
                    })
                    .register();
                proxies
                    .borrow_mut()
                    .add_proxy(Box::new(proxy), vec![Box::new(listener)]);
                if only_existent {
                    scheduler.schedule_roundtrip();
                }
            }
        })
        .register();

    if only_existent {
        scheduler.schedule_roundtrip();
        scheduler.run_until_sync();
    } else {
        scheduler.run();
    }

    drop(reg_listener);
    drop(proxies);
}

pub fn iterate_objects<F>(
    mainloop: &MainLoopRc,
    core: &CoreRc,
    registry: &RegistryRc,
    object_callback: F,
    only_existent: bool,
) where
    F: Fn(&GlobalObject<&DictRef>) -> bool + 'static,
{
    let reg_listener = registry
        .add_listener_local()
        .global({
            let mainloop = mainloop.clone();
            move |global| {
                if object_callback(global) {
                    mainloop.quit();
                }
            }
        })
        .register();

    if only_existent {
        do_roundtrip(mainloop, core);
    } else {
        mainloop.run();
    }

    drop(reg_listener);
}
/// Do a single roundtrip to process all events.
/// See the example in roundtrip.rs for more details on this.
pub fn do_roundtrip(mainloop: &MainLoopRc, core: &Core) {
    let done = Rc::new(Cell::new(false));
    let done_clone = done.clone();
    let loop_clone = mainloop.clone();

    // Trigger the sync event. The server's answer won't be processed until we start the main loop,
    // so we can safely do this before setting up a callback. This lets us avoid using a Cell.
    let pending = core.sync(0).expect("sync failed");

    let _listener_core = core
        .add_listener_local()
        .done(move |id, seq| {
            if id == PW_ID_CORE && seq == pending {
                done_clone.set(true);
                loop_clone.quit();
            }
        })
        .register();

    while !done.get() {
        mainloop.run();
    }
}

pub fn create_node(node_name: impl AsRef<str>, core: &Core) -> Result<Node, pipewire::Error> {
    core.create_object::<Node>(
        "adapter",
        &properties! {
            *keys::FACTORY_NAME => "support.null-audio-sink",
            *keys::NODE_NAME => node_name.as_ref(),
            *keys::MEDIA_CLASS => "Audio/Source/Virtual",
            "audio.position" => "[FL, FR]",
            *keys::OBJECT_LINGER => "1",
        },
    )
}

pub fn link_ports(
    output: &Ports,
    input: &Ports,
    core: &Core,
) -> Result<[Link; 2], pipewire::Error> {
    println!("Linking {output:?} with {input:?}");

    let links = [
        (input.fl_port, output.fl_port),
        (input.fr_port, output.fr_port),
    ]
    .map(|(input, output)| {
        core.create_object::<Link>(
            "link-factory",
            &properties! {
                *keys::FACTORY_NAME => "support.null-audio-sink",
                *keys::LINK_INPUT_PORT => input.to_string(),
                *keys::LINK_OUTPUT_PORT => output.to_string(),
                *keys::OBJECT_LINGER => "1",
            },
        )
    });

    Ok(links
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .unwrap())
}

pub fn await_node_creation(node: Node, mainloop: &MainLoopRc, core: &Core) -> u32 {
    let node_id = Rc::new(OnceCell::new());

    let listener = node
        .upcast()
        .add_listener_local()
        .bound({
            let node_id = node_id.clone();
            move |id| node_id.set(id).unwrap()
        })
        .register();

    do_roundtrip(mainloop, core);

    drop(listener);

    let node_id = *node_id.get().unwrap();

    return node_id;
}

#[derive(Debug, Clone, Copy)]
pub struct Ports {
    pub fl_port: u32,
    pub fr_port: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct MaybePorts {
    pub fl_port: Option<u32>,
    pub fr_port: Option<u32>,
}

impl MaybePorts {
    pub fn both(self) -> Option<Ports> {
        let MaybePorts { fl_port, fr_port } = self;
        let Some(fl_port) = fl_port else {
            return None;
        };
        let Some(fr_port) = fr_port else {
            return None;
        };
        Some(Ports { fl_port, fr_port })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PortDirection {
    INPUT,
    OUTPUT,
}

impl PortDirection {
    fn to_pipewire_string(&self) -> &'static str {
        match self {
            PortDirection::INPUT => "in",
            PortDirection::OUTPUT => "out",
        }
    }
}

struct PortInfo<'a> {
    channel: &'a str,
    id: u32,
}

fn is_global_a_port_for_node<'a>(
    global: &'a GlobalObject<&'a DictRef>,
    direction: &PortDirection,
    node_id: u32,
) -> Option<PortInfo<'a>> {
    let direction_name = direction.to_pipewire_string();

    let Some(ref props) = global.props else {
        return None;
    };
    if global.type_ == ObjectType::Port
        && props.get(*keys::NODE_ID) == Some(&node_id.to_string())
        && props.get(*keys::PORT_DIRECTION) == Some(direction_name)
    {
        let port_id: u32 = global.id;
        let Some(audio_channel) = props.get(*keys::AUDIO_CHANNEL) else {
            return None;
        };

        Some(PortInfo {
            channel: audio_channel,
            id: port_id,
        })
    } else {
        None
    }
}

fn find_fl_fr_ports_internal(
    node_id: u32,
    direction: PortDirection,
    mainloop: &MainLoopRc,
    core: &CoreRc,
    registry: &RegistryRc,
    only_existent: bool,
) -> MaybePorts {
    let fl_port = Rc::new(OnceCell::new());
    let fr_port = Rc::new(OnceCell::new());

    iterate_objects(
        &mainloop,
        &core,
        &registry,
        {
            let fl_port = fl_port.clone();
            let fr_port = fr_port.clone();
            move |global| {
                if let Some(port) = is_global_a_port_for_node(global, &direction, node_id) {
                    // Save port id into channel cell
                    if let Some(channel_cell) = match port.channel {
                        "FL" => Some(&fl_port),
                        "FR" => Some(&fr_port),
                        _ => None,
                    } {
                        channel_cell
                            .set(port.id)
                            .expect("Node has multiple ports for same channel");
                    }

                    // Stop searching if we found all ports
                    let found_all_ports = [&fl_port, &fr_port]
                        .iter()
                        .all(|port_id| port_id.get().is_some());

                    return found_all_ports;
                } else {
                    false
                }
            }
        },
        only_existent,
    );

    let fl_port = Rc::try_unwrap(fl_port).unwrap().into_inner();
    let fr_port = Rc::try_unwrap(fr_port).unwrap().into_inner();

    MaybePorts { fl_port, fr_port }
}

pub fn await_find_fl_fr_ports(
    node_id: u32,
    direction: PortDirection,
    mainloop: &MainLoopRc,
    core: &CoreRc,
    registry: &RegistryRc,
) -> Ports {
    // We can unwrap, it blocks until it finds both channels
    find_fl_fr_ports_internal(node_id, direction, mainloop, core, registry, false)
        .both()
        .unwrap()
}

pub fn find_fl_fr_ports(
    node_id: u32,
    direction: PortDirection,
    mainloop: &MainLoopRc,
    core: &CoreRc,
    registry: &RegistryRc,
) -> Option<Ports> {
    find_fl_fr_ports_internal(node_id, direction, mainloop, core, registry, true).both()
}

pub fn monitor_nodes<F>(
    on_node_added: F,
    mainloop: &MainLoopRc,
    core: &CoreRc,
    registry: &RegistryRc,
) where
    F: Fn(u32) + Clone + 'static,
{
    iterate_nodes(
        &mainloop,
        &core,
        &registry,
        move |node| {
            let Some(ref props) = node.props() else {
                return false;
            };
            // TODO: Add exclusions
            if props.get(*keys::MEDIA_CLASS) == Some("Stream/Output/Audio") {
                let node_id: u32 = node.id();

                on_node_added(node_id);
            };
            false
        },
        false,
    );
}

pub fn connect_node(
    node: u32,
    target_ports: &Ports,
    mainloop: &MainLoopRc,
    core: &CoreRc,
    registry: &RegistryRc,
) -> Result<(), Box<dyn std::error::Error>> {
    let ports = await_find_fl_fr_ports(node, PortDirection::OUTPUT, &mainloop, &core, &registry);

    link_ports(&ports, target_ports, core)?;
    do_roundtrip(mainloop, core);

    Ok(())
}

pub fn find_node_by_name(
    mainloop: &MainLoopRc,
    core: &CoreRc,
    registry: &RegistryRc,
    node_name: impl AsRef<str> + 'static,
) -> Option<u32> {
    let result = Rc::new(OnceCell::new());
    iterate_nodes(
        mainloop,
        core,
        registry,
        {
            let result = result.clone();
            let node_name = Rc::new(node_name);
            move |node| {
                if node
                    .props()
                    .and_then(|props| props.get(*pipewire::keys::NODE_NAME))
                    .is_some_and(|name| node_name.deref().as_ref() == name)
                {
                    result.set(node.id());
                    true
                } else {
                    false
                }
            }
        },
        true,
    );
    return Rc::try_unwrap(result).unwrap().into_inner();
}
