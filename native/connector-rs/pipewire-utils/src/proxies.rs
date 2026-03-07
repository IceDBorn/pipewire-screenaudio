use std::collections::HashMap;

use pipewire::proxy::{Listener, ProxyT};

pub struct ProxyRef {
    _proxy: Box<dyn ProxyT>,
    _listeners: Vec<Box<dyn Listener>>,
}

#[derive(Default)]
pub struct ProxyRefs {
    refs: HashMap<u32, ProxyRef>,
}

impl ProxyRefs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_proxy(&mut self, proxy: Box<dyn ProxyT>, listeners: Vec<Box<dyn Listener>>) {
        let proxy_id = {
            let proxy = proxy.upcast_ref();
            proxy.id()
        };

        self.refs.insert(
            proxy_id,
            ProxyRef {
                _proxy: proxy,
                _listeners: listeners,
            },
        );
    }

    pub fn remove_proxy(&mut self, proxy_id: &u32) -> bool {
        self.refs.remove(proxy_id).is_some()
    }
}
