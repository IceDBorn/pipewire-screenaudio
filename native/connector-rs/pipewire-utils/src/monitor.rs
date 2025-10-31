use std::{
    cell::{Cell, RefCell},
    collections::{hash_map::Entry, HashMap, HashSet},
    rc::Rc,
};

use pipewire::{
    link::Link,
    proxy::{ProxyListener, ProxyT},
};
use tracing::instrument;

use crate::pipewire_objects::{OwnedPortInfo, PortInfo};

#[derive(Clone)]
pub struct NodeAndPortsRegistry<AddedCallback, RemovedCallback> {
    relevant_nodes: Rc<RefCell<HashSet<u32>>>,
    nodes_to_ports: Rc<RefCell<HashMap<u32, HashSet<u32>>>>,
    ports: Rc<RefCell<HashMap<u32, OwnedPortInfo>>>,
    relevant_port_added_callback: Option<AddedCallback>,
    relevant_port_removed_callback: Option<RemovedCallback>,
}

impl<AddedCallback, RemovedCallback> Default
    for NodeAndPortsRegistry<AddedCallback, RemovedCallback>
{
    fn default() -> Self {
        Self {
            relevant_nodes: Rc::new(RefCell::new(HashSet::new())),
            nodes_to_ports: Rc::new(RefCell::new(HashMap::<u32, HashSet<u32>>::new())),
            ports: Rc::new(RefCell::new(HashMap::<u32, OwnedPortInfo>::new())),
            relevant_port_added_callback: None,
            relevant_port_removed_callback: None,
        }
    }
}

impl<AddedCallback: Fn(&OwnedPortInfo), RemovedCallback: Fn(&OwnedPortInfo)>
    NodeAndPortsRegistry<AddedCallback, RemovedCallback>
{
    pub fn set_relevant_port_added_callback(&mut self, callback: AddedCallback) {
        self.relevant_port_added_callback = Some(callback);
    }

    pub fn set_relevant_port_removed_callback(&mut self, callback: RemovedCallback) {
        self.relevant_port_removed_callback = Some(callback);
    }

    fn invoke_for_all_ports_in_node<F: Fn(&OwnedPortInfo)>(&self, node_id: u32, callback: F) {
        if let Some(port_ids) = self.nodes_to_ports.borrow().get(&node_id) {
            let ports = self.ports.borrow();
            for port_id in port_ids {
                let port_info = ports.get(port_id).expect("there must be info");

                callback(port_info);
            }
        }
    }

    pub fn is_node_relevant(&self, node_id: u32) -> bool {
        self.relevant_nodes.borrow().contains(&node_id)
    }

    #[instrument(skip(self))]
    pub fn add_relevant_node(&self, node_id: u32) {
        self.relevant_nodes.borrow_mut().insert(node_id);

        if let Some(callback) = &self.relevant_port_added_callback {
            self.invoke_for_all_ports_in_node(node_id, callback);
        }
    }

    #[instrument(skip(self))]
    pub fn try_remove_relevant_node(&self, node_id: u32) -> bool {
        let removed = self.relevant_nodes.borrow_mut().remove(&node_id);

        if removed {
            if let Some(callback) = &self.relevant_port_removed_callback {
                self.invoke_for_all_ports_in_node(node_id, callback);
            }
        }

        removed
    }

    #[instrument(skip(self))]
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
            tracing::trace!(port_id = port_info.id, "port is from a relevant node");
            if let Some(callback) = &self.relevant_port_added_callback {
                callback(&port_info);
            }
        }

        self.ports.borrow_mut().insert(port_info.id, port_info);
    }

    #[instrument(skip(self))]
    pub fn try_remove_port(&self, port_id: u32) -> Option<OwnedPortInfo> {
        let port_info = self.ports.borrow_mut().remove(&port_id);
        if let Some(port_info) = port_info.as_ref() {
            if self.relevant_nodes.borrow().contains(&port_info.node_id) {
                if let Some(callback) = &self.relevant_port_removed_callback {
                    callback(port_info);
                }
            }
            self.nodes_to_ports
                .borrow_mut()
                .get_mut(&port_info.node_id)
                .unwrap()
                .remove(&port_id);
        }
        port_info
    }
}

pub struct LinkTrackerHandle {
    #[expect(unused)]
    listener: ProxyListener,
    id: Rc<Cell<Option<u32>>>,
}

impl LinkTrackerHandle {
    pub fn new(link: Link) -> Self {
        let id = Rc::new(Cell::new(None));
        let listener = link
            .upcast()
            .add_listener_local()
            .bound({
                let id = id.clone();
                move |link_id| {
                    id.set(Some(link_id));
                }
            })
            .register();
        Self { listener, id }
    }

    pub fn id(&self) -> Option<u32> {
        self.id.get()
    }
}
