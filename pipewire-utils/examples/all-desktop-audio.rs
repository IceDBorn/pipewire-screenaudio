use std::{
    error::Error,
    sync::mpsc::{self, Sender},
    thread,
};

use pipewire::{Context, Core, MainLoop};

use pipewire_utils::{
    self, await_find_fl_fr_ports, await_node_creation, create_node, do_roundtrip, link_ports, Ports,
};

fn monitor_nodes(node_channel: Sender<u32>) -> Result<(), Box<dyn Error>> {
    let mainloop = MainLoop::new()?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;
    let registry = core.get_registry()?;
    pipewire_utils::monitor_nodes(node_channel, &mainloop, &registry);

    Ok(())
}

struct NodeWithPorts {
    id: u32,
    ports: Ports,
}

fn create_virtmic_node(
    mainloop: &MainLoop,
    core: &Core,
) -> Result<NodeWithPorts, Box<dyn std::error::Error>> {
    let node = create_node("pipewire-screenaudio", core).expect("Failed to create node");

    let node_id = await_node_creation(node, mainloop, core);
    dbg!(node_id);

    let registry = core.get_registry()?;
    let ports = await_find_fl_fr_ports(
        node_id,
        pipewire_utils::PortDirection::INPUT,
        &mainloop,
        &registry,
    );

    return Ok(NodeWithPorts { id: node_id, ports });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mainloop = MainLoop::new()?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;

    let virtmic = create_virtmic_node(&mainloop, &core)?;

    let (nodes_tx, nodes_rx) = mpsc::channel();

    thread::spawn(move || {
        monitor_nodes(nodes_tx).unwrap();
    });

    for node in nodes_rx {
        dbg!(node);
        let registry = core.get_registry()?;
        let ports = await_find_fl_fr_ports(
            node,
            pipewire_utils::PortDirection::OUTPUT,
            &mainloop,
            &registry,
        );

        dbg!(ports);
        link_ports(&ports, &virtmic.ports, &core)?;
        do_roundtrip(&mainloop, &core);
    }

    core.get_registry()?.destroy_global(virtmic.id);
    do_roundtrip(&mainloop, &core);

    Ok(())
}
