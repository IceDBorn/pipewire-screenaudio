use std::{
    cell::{Cell, OnceCell},
    rc::Rc,
};

use pipewire::{
    keys,
    node::Node,
    properties,
    proxy::ProxyT,
    registry::{GlobalObject, Registry},
    spa::{ForeignDict, ReadableDict},
    types::ObjectType,
    Core, MainLoop, PW_ID_CORE,
};

pub fn iterate_existing_objects<F>(
    mainloop: &MainLoop,
    core: &Core,
    registry: &Registry,
    object_callback: F,
) where
    F: Fn(&GlobalObject<ForeignDict>) -> bool + 'static,
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

    do_roundtrip(mainloop, core);

    drop(reg_listener);
}

pub fn iterate_objects<F>(mainloop: &MainLoop, registry: &Registry, object_callback: F)
where
    F: Fn(&GlobalObject<ForeignDict>) -> bool + 'static,
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

    mainloop.run();

    drop(reg_listener);
}

/// Do a single roundtrip to process all events.
/// See the example in roundtrip.rs for more details on this.
pub fn do_roundtrip(mainloop: &MainLoop, core: &Core) {
    let done = Rc::new(Cell::new(false));
    let done_clone = done.clone();
    let loop_clone = mainloop.clone();

    // Trigger the sync event. The server's answer won't be processed until we start the main loop,
    // so we can safely do this before setting up a callback. This lets us avoid using a Cell.
    let pending = core.sync(0).expect("sync failed");

    let _listener_core = core
        .add_listener_local()
        .done(move |id, seq| {
            //if id == PW_ID_CORE {
            //println!("{seq:?}");
            //println!("{pending:?}");
            //}
            if id == PW_ID_CORE && seq == pending {
                //println!("AAAAAAAAAAAAAAAA");
                done_clone.set(true);
                loop_clone.quit();
            }
        })
        .register();

    while !done.get() {
        mainloop.run();
    }
}

pub fn create_node(node_name: &str, core: &Core) -> Result<Node, pipewire::Error> {
    core.create_object::<Node, _>(
        "adapter",
        &properties! {
            *keys::FACTORY_NAME => "support.null-audio-sink",
            *keys::NODE_NAME => node_name,
            *keys::MEDIA_CLASS => "Audio/Source/Virtual",
            "audio.position" => "[FL, FR]",
            *keys::OBJECT_LINGER => "1",
        },
    )
}

pub fn await_node_creation(node: Node, mainloop: &MainLoop, core: &Core) -> u32 {
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

pub struct Ports {
    pub fl_port: u32,
    pub fr_port: u32,
}

pub fn await_find_fl_fr_input_ports(node_id: u32, mainloop: &MainLoop, registry: &Registry) -> Ports {
    let fl_port = Rc::new(OnceCell::new());
    let fr_port = Rc::new(OnceCell::new());

    iterate_objects(&mainloop, &registry, {
        let fl_port = fl_port.clone();
        let fr_port = fr_port.clone();
        move |global| {
            let Some(ref props) = global.props else {
            return false;
        };
            if global.type_ == ObjectType::Port
                && props.get(*keys::NODE_ID) == Some(&node_id.to_string())
                && props.get(*keys::PORT_DIRECTION) == Some("out")
            {
                let port_id: u32 = global.id;
                let Some(audio_channel) = props.get(*keys::AUDIO_CHANNEL) else {
                    return false;
                };

                // Save port id into channel cell
                if let Some(channel_cell) = match audio_channel {
                    "FL" => Some(&fl_port),
                    "FR" => Some(&fr_port),
                    _ => None,
                } {
                    channel_cell
                        .set(port_id)
                        .expect("Node has multiple ports for same channel");
                }

                // Stop searching if we found all ports
                let found_all_ports = [&fl_port, &fr_port]
                    .iter()
                    .all(|port_id| port_id.get().is_some());

                return found_all_ports;
            };
            false
        }
    });

    // We can unwrap because iterate_objects blocks until it finds both channels
    let fl_port = *fl_port.get().unwrap();
    let fr_port = *fr_port.get().unwrap();

    Ports { fl_port, fr_port }
}

