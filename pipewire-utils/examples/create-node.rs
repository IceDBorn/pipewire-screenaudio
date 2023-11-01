use pipewire::{Context, MainLoop};

use pipewire_utils::{
    self, create_node, await_find_fl_fr_input_ports, await_node_creation, Ports,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mainloop = MainLoop::new()?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;

    let node = create_node("pipewire-screenaudio", &core).expect("Failed to create node");

    let node_id = await_node_creation(node, &mainloop, &core);
    dbg!(node_id);

    let registry = core.get_registry()?;
    let Ports { fl_port, fr_port } = await_find_fl_fr_input_ports(node_id, &mainloop, &registry);
    dbg!(fl_port, fr_port);

    Ok(())
}
