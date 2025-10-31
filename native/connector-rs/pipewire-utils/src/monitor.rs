use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap, HashSet},
    rc::Rc,
};

use crate::pipewire_objects::{OwnedPortInfo, PortInfo};

#[derive(Clone)]
pub struct NodeAndPortsRegistry<Callback> {
    relevant_nodes: Rc<RefCell<HashSet<u32>>>,
    nodes_to_ports: Rc<RefCell<HashMap<u32, HashSet<u32>>>>,
    ports: Rc<RefCell<HashMap<u32, OwnedPortInfo>>>,
    relevant_port_callback: Option<Callback>,
}

impl<Callback> Default for NodeAndPortsRegistry<Callback> {
    fn default() -> Self {
        Self {
            relevant_nodes: Rc::new(RefCell::new(HashSet::new())),
            nodes_to_ports: Rc::new(RefCell::new(HashMap::<u32, HashSet<u32>>::new())),
            ports: Rc::new(RefCell::new(HashMap::<u32, OwnedPortInfo>::new())),
            relevant_port_callback: None,
        }
    }
}

impl<Callback: Fn(&OwnedPortInfo) + Clone + 'static> NodeAndPortsRegistry<Callback> {
    pub fn set_relevant_port_callback(&mut self, callback: Callback) {
        self.relevant_port_callback = Some(callback);
    }

    pub fn add_relevant_node(&self, node_id: u32) {
        self.relevant_nodes.borrow_mut().insert(node_id);

        if let Some(port_ids) = self.nodes_to_ports.borrow().get(&node_id) {
            let ports = self.ports.borrow();
            for port_id in port_ids {
                let port_info = ports.get(port_id).expect("there must be info");

                if let Some(relevant_port_callback) = &self.relevant_port_callback {
                    relevant_port_callback(port_info);
                }
            }
        }
    }

    pub fn remove_relevant_node(&self, node_id: u32) {
        self.relevant_nodes.borrow_mut().remove(&node_id);
    }

    pub fn add_port(&self, port_info: PortInfo<'_>) {
        match self.nodes_to_ports.borrow_mut().entry(port_info.node_id) {
            Entry::Vacant(entry) => {
                entry.insert(HashSet::from_iter([port_info.id]));
            }
            Entry::Occupied(mut entry) => {
                let node_ports = entry.get_mut();
                node_ports.insert(port_info.id);
            }
        }

        let port_info = OwnedPortInfo::from(port_info);

        if self.relevant_nodes.borrow().contains(&port_info.node_id) {
            tracing::debug!(port_id = port_info.id, "port is from a relevant node");
            if let Some(relevant_port_callback) = &self.relevant_port_callback {
                relevant_port_callback(&port_info);
            }
        }

        self.ports.borrow_mut().insert(port_info.id, port_info);
    }

    pub fn try_remove_port(&self, port_id: u32) -> Option<OwnedPortInfo> {
        let port_info = self.ports.borrow_mut().remove(&port_id);
        if let Some(port_info) = port_info.as_ref() {
            self.nodes_to_ports
                .borrow_mut()
                .get_mut(&port_info.node_id)
                .unwrap()
                .remove(&port_id);
        }
        port_info
    }
}
