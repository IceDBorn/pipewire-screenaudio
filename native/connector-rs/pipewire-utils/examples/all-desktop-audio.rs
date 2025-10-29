use std::error::Error;

use pipewire::{
    context::ContextRc,
    core::{Core, CoreRc},
    loop_::Signal,
    main_loop::MainLoopRc,
};

use pipewire_utils::{
    self, await_find_fl_fr_ports, await_node_creation, create_node, do_roundtrip, link_ports, Ports,
};

struct NodeWithPorts {
    id: u32,
    ports: Ports,
}

fn create_virtmic_node(
    mainloop: &MainLoopRc,
    core: &CoreRc,
) -> Result<NodeWithPorts, Box<dyn std::error::Error>> {
    let node = create_node("pipewire-screenaudio", core).expect("Failed to create node");

    let node_id = await_node_creation(node, mainloop, core);
    dbg!(node_id);

    let ports = await_find_fl_fr_ports(
        node_id,
        pipewire_utils::PortDirection::INPUT,
        mainloop,
        core,
    );

    Ok(NodeWithPorts { id: node_id, ports })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mainloop = MainLoopRc::new(None)?;

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
    let context = ContextRc::new(&mainloop, None)?;
    let core = context.connect_rc(None)?;

    let virtmic = create_virtmic_node(&mainloop, &core)?;

    // Start monitoring new nodes
    pipewire_utils::monitor_nodes(
        {
            let virtmic_ports = virtmic.ports;
            let mainloop = MainLoopRc::new(None)?;
            let context = ContextRc::new(&mainloop, None)?;
            move |node| {
                // Moving this line outside of the closure causes a SIGSEGV
                let core = context.connect_rc(None).unwrap();
                dbg!(node);
                pipewire_utils::connect_node(node, &virtmic_ports, &mainloop, &core).unwrap();
            }
        },
        &mainloop,
        &core,
    );

    // Destroy virtmic node
    core.get_registry()?.destroy_global(virtmic.id);
    do_roundtrip(&mainloop, &core);

    Ok(())
}
