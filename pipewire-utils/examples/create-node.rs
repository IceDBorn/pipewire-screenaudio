use pipewire::{Context, MainLoop};

use pipewire_utils::{self, await_find_fl_fr_ports, await_node_creation, create_node, Ports};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mainloop = MainLoop::new()?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;

    let node = create_node("pipewire-screenaudio", &core).expect("Failed to create node");

    let node_id = await_node_creation(node, &mainloop, &core);
    dbg!(node_id);

    let registry = core.get_registry()?;
    let Ports { fl_port, fr_port } = await_find_fl_fr_ports(
        node_id,
        pipewire_utils::PortDirection::INPUT,
        &mainloop,
        &registry,
    );
    dbg!(fl_port, fr_port);

    Ok(())
}
