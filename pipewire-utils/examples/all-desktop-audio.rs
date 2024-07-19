use std::error::Error;

use pipewire::{context::Context, core::Core, main_loop::MainLoop, loop_::Signal};

use pipewire_utils::{
    self, await_find_fl_fr_ports, await_node_creation, create_node, do_roundtrip, link_ports, Ports,
};

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

fn connect_node(
    node: u32,
    virtmic_ports: &Ports,
    mainloop: &MainLoop,
    core: &Core,
) -> Result<(), Box<dyn Error>> {
    let registry = core.get_registry()?;
    let ports = await_find_fl_fr_ports(
        node,
        pipewire_utils::PortDirection::OUTPUT,
        &mainloop,
        &registry,
    );

    dbg!(ports);
    link_ports(&ports, &virtmic_ports, core)?;
    do_roundtrip(mainloop, core);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mainloop = MainLoop::new(None)?;

    // Make SIGINT stop mainloop
    let _sig_int = mainloop.loop_().add_signal_local(Signal::SIGINT, {
        let mainloop = mainloop.downgrade();
        move || {
            if let Some(mainloop) = mainloop.upgrade() {
                mainloop.quit();
            }
        }
    });

    // Connect with Pipewire
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;

    let virtmic = create_virtmic_node(&mainloop, &core)?;

    // Start monitoring new nodes
    let registry = core.get_registry()?;
    pipewire_utils::monitor_nodes(
        {
            let virtmic_ports = virtmic.ports;
            let mainloop = MainLoop::new(None)?;
            let context = Context::new(&mainloop)?;
            move |node| {
                // Moving this line outside of the closure causes a SIGSEGV
                let core = context.connect(None).unwrap();
                dbg!(node);
                connect_node(node, &virtmic_ports, &mainloop, &core).unwrap();
            }
        },
        &mainloop,
        &registry,
    );

    // Destroy virtmic node
    core.get_registry()?.destroy_global(virtmic.id);
    do_roundtrip(&mainloop, &core);

    Ok(())
}
