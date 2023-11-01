use std::{
    error::Error,
    sync::mpsc::{self, Sender},
    thread,
};

use pipewire::{Context, MainLoop};

use pipewire_utils::{self, await_find_fl_fr_ports};

fn monitor_nodes(node_channel: Sender<u32>) -> Result<(), Box<dyn Error>> {
    let mainloop = MainLoop::new()?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;
    let registry = core.get_registry()?;
    pipewire_utils::monitor_nodes(node_channel, &mainloop, &registry);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (nodes_tx, nodes_rx) = mpsc::channel();

    thread::spawn(move || {
        monitor_nodes(nodes_tx).unwrap();
    });

    let mainloop = MainLoop::new()?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;
    for node in nodes_rx {
        let registry = core.get_registry()?;
        let ports = await_find_fl_fr_ports(
            node,
            pipewire_utils::PortDirection::OUTPUT,
            &mainloop,
            &registry,
        );

        dbg!(ports.fl_port, ports.fr_port);
    }

    Ok(())
}
