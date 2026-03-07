use crate::{pipewire_client::PipewireClient, pipewire_objects::NodeWithPorts, PipewireError};

pub struct ManagedNode {
    pipewire_client: PipewireClient,
    node_with_ports: NodeWithPorts,
}

impl ManagedNode {
    pub fn create_managed_node(
        pipewire_client: &PipewireClient,
        node_name: impl AsRef<str>,
    ) -> Result<Self, PipewireError> {
        let node = pipewire_client.await_create_node(node_name)?;

        Ok(ManagedNode {
            pipewire_client: pipewire_client.clone(),
            node_with_ports: node,
        })
    }

    pub fn get_node_with_ports(&self) -> &NodeWithPorts {
        &self.node_with_ports
    }
}

impl Drop for ManagedNode {
    fn drop(&mut self) {
        let id = self.node_with_ports.id;
        if let Err(err) = self.pipewire_client.destroy_global(id) {
            tracing::error!("error removing managed node {id}, ignoring error: {err}");
        }
    }
}
