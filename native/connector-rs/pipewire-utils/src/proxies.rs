use std::collections::{HashMap, HashSet};

use pipewire::proxy::{Listener, ProxyT};

pub struct ProxyRef {
    proxy: Box<dyn ProxyT>,
    listeners: Vec<Box<dyn Listener>>,
}

#[derive(Default)]
pub struct ProxyRefs {
    refs: HashMap<u32, ProxyRef>,
}

impl ProxyRefs {
    fn new() -> Self {
        Self::default()
    }

    pub fn add_proxy(&mut self, proxy: Box<dyn ProxyT>, listeners: Vec<Box<dyn Listener>>) {
        let proxy_id = {
            let proxy = proxy.upcast_ref();
            proxy.id()
        };

        self.refs.insert(proxy_id, ProxyRef { proxy, listeners });
    }

    pub fn remove_proxy(&mut self, proxy_id: &u32) {
        self.refs.remove(proxy_id);
    }
}
