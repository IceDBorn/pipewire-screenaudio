use pipewire_utils::PipewireClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PipewireClient::new()?;

    let node = client.await_create_node("pipewire-screenaudio")?;
    dbg!(node);

    Ok(())
}
