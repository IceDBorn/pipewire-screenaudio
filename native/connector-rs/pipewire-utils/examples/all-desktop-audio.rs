use pipewire_utils::{
    self, cancellation_signal::CancellationSignal, utils::ManagedNode, PipewireClient,
};
use tracing::Level;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let client = PipewireClient::new()?;
    let node = ManagedNode::create_managed_node(&client, "test node")?;
    tracing::info!("created node");

    let (controller, signal) = CancellationSignal::pair();
    ctrlc::set_handler(move || {
        controller.cancel();
    })
    .unwrap();

    tracing::info!("started monitoring");
    client.monitor_and_connect_nodes(node.get_node_with_ports().ports, signal, |_| true)?;

    tracing::info!("finishing");

    Ok(())
}
