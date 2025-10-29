use std::{cell::RefCell, collections::HashSet, rc::Rc, thread, time::Duration};

extern crate serde_json;
use pipewire::{
  channel::Receiver,
  context::ContextRc,
  core::CoreRc,
  loop_::Signal,
  main_loop::MainLoopRc,
  node::NodeInfoRef,
  properties::PropertiesBox,
  registry::{GlobalObject, Registry, RegistryBox, RegistryRc},
  spa::{support::system::IoFlags, utils::dict::DictRef},
  types::ObjectType,
};
use pipewire_utils::{iterate_objects, iterate_objects_scheduled, Ports};
use serde::Serialize;
use serde_json::{json, Deserializer, Map, Value};
use thiserror::Error;

extern crate log;
use log::debug;

use crate::helpers::JsonGetters;

#[derive(Error, Debug)]
pub enum Error {
  #[error("internal pipewire error: {0}")]
  PipewireError(pipewire::Error),
}

impl From<pipewire::Error> for Error {
  fn from(value: pipewire::Error) -> Self {
    Self::PipewireError(value)
  }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

fn get_node_name(node: &Value) -> Result<String, String> {
  let result = node.get_fields_chain(vec!["info", "props", "node.name"]);
  match result {
    Ok(v) => Ok(v.as_str().unwrap().to_string()),
    Err(e) => Err(e),
  }
}

#[derive(Debug, Serialize)]
pub struct NodeProperties {
  #[serde(rename = "application.name")]
  application_name: Option<String>,
  #[serde(rename = "media.name")]
  media_name: String,
  #[serde(rename = "object.serial")]
  object_serial: i64,

  #[serde(skip_serializing)]
  media_class: String,
}

#[derive(Debug, Serialize)]
pub struct OutputNode {
  id: u32,
  properties: NodeProperties,
}

impl OutputNode {
  fn from_node_ref(node: &NodeInfoRef) -> Result<Self, String> {
    let Some(props) = node.props() else {
      return Err("missing props".to_owned());
    };
    let application_name = props.get("application.name");
    let Some(object_serial) = props.get("object.serial") else {
      return Err("missing object.serial".to_owned());
    };
    let Some(media_class) = props.get("media.class") else {
      return Err("missing media.class".to_owned());
    };
    let Some(media_name) = props.get("media.name") else {
      return Err("missing media.name".to_owned());
    };
    Ok(OutputNode {
      id: node.id(),
      properties: {
        NodeProperties {
          application_name: application_name.map(|s| s.to_owned()),
          media_name: media_name.to_owned(),
          object_serial: object_serial.parse().unwrap(),
          media_class: media_class.to_owned(),
        }
      },
    })
  }
}

pub fn get_output_nodes() -> Result<Vec<OutputNode>> {
  let mainloop = MainLoopRc::new(None).map_err(Error::from)?;
  let context = ContextRc::new(&mainloop, None).map_err(Error::from)?;
  let mut connect_props = PropertiesBox::new();
  connect_props.insert(pipewire::keys::REMOTE_INTENTION.to_owned(), "manager");
  let core = context
    .connect_rc(Some(connect_props))
    .map_err(Error::from)?;

  let output_nodes = Rc::new(RefCell::new(vec![]));
  pipewire_utils::iterate_nodes(
    &mainloop,
    &core,
    {
      let output_nodes = output_nodes.clone();
      move |node| {
        match OutputNode::from_node_ref(node) {
          Ok(node) => {
            if node.properties.media_class == "Stream/Output/Audio" {
              output_nodes.borrow_mut().push(node);
            } else {
              debug!("node {} is not an output", node.id);
            }
          }
          Err(err) => {
            log::trace!(
              "node {} does not contain all required properties: {}",
              node.id(),
              err
            )
          }
        }
        false
      }
    },
    true,
  );

  return Ok(Rc::into_inner(output_nodes).unwrap().into_inner());
}

pub fn get_node_id_from_serial(serial: i64) -> Option<u32> {
  let dump = get_output_nodes().unwrap();
  log::debug!("nodes: {dump:?}");
  let result = dump
    .into_iter()
    .find(|node| node.properties.object_serial == serial);

  if result.is_some() {
    log::info!("Found Target: {:?}", result.as_ref().unwrap());
  }

  result.map(|v| v.id)
}

pub fn connect_nodes(
  mainloop: &MainLoopRc,
  core: &CoreRc,
  in_node_id: u32,
  mic_ports: &Ports,
) -> bool {
  pipewire_utils::connect_node(in_node_id, mic_ports, mainloop, core).is_ok()
}

fn remove_links(mainloop: &MainLoopRc, core: &CoreRc, port_ids: HashSet<u32>) {
  log::debug!("removing links to: {:?}", &port_ids);
  let is_link_to_port = move |global: &GlobalObject<&DictRef>| {
    if global.type_ != ObjectType::Link {
      return false;
    }
    let Some(props) = global.props else {
      return false;
    };
    let Some(output_port) = props.get("link.input.port") else {
      return false;
    };
    let Ok(output_port) = output_port.parse::<u32>() else {
      return false;
    };
    port_ids.contains(&output_port)
  };
  iterate_objects_scheduled(mainloop, core, {
    let registry = core.get_registry_rc().unwrap();
    move |scheduler, global| {
      if is_link_to_port(global) {
        log::debug!("removing link: {}", global.id);
        registry.destroy_global(global.id);
        scheduler.schedule_roundtrip();
      }
      false
    }
  });
  log::debug!("links removed");
}

pub fn disconnect_node(mainloop: &MainLoopRc, core: &CoreRc, mic_ports: &Ports) {
  remove_links(
    mainloop,
    core,
    HashSet::from_iter([mic_ports.fl_port, mic_ports.fr_port]),
  );
}

#[derive(Debug)]
pub struct TerminateSignal;

pub fn monitor_and_connect_nodes(
  mic_ports: Ports,
  stop_signal_receiver: Receiver<TerminateSignal>,
) -> Result<(), pipewire::Error> {
  let mainloop = MainLoopRc::new(None)?;
  let context = ContextRc::new(&mainloop, None)?;
  let core = context.connect_rc(None)?;
  let _receiver = stop_signal_receiver.attach(mainloop.loop_(), {
    let mainloop = mainloop.clone();
    move |_| mainloop.quit()
  });
  log::info!("starting node monitoring loop");
  pipewire_utils::monitor_nodes(
    {
      let mainloop = MainLoopRc::new(None)?;
      let context = ContextRc::new(&mainloop, None)?;
      move |node| {
        log::info!("connecting to node {node}");
        let core = context.connect_rc(None).unwrap();
        pipewire_utils::connect_node(node, &mic_ports, &mainloop, &core);
      }
    },
    &mainloop,
    &core,
  );
  log::info!("exited node monitoring loop");
  Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct NodeWithPorts {
  pub id: u32,
  pub ports: Ports,
}

impl NodeWithPorts {
  pub fn create_virtual_source(
    mainloop: &MainLoopRc,
    core: &CoreRc,
    node_name: impl AsRef<str>,
  ) -> Result<Self, Box<dyn std::error::Error>> {
    let node = pipewire_utils::create_node(&node_name, core).expect("Failed to create node");

    let node_id = pipewire_utils::await_node_creation(node, mainloop, core);
    dbg!(node_id);

    let ports = pipewire_utils::await_find_fl_fr_ports(
      node_id,
      pipewire_utils::PortDirection::INPUT,
      mainloop,
      core,
    );

    return Ok(NodeWithPorts { id: node_id, ports });
  }
}

pub struct ManagedNode {
  node_with_ports: NodeWithPorts,
  mainloop: MainLoopRc,
  context: ContextRc,
  core: CoreRc,
  registry: RegistryRc,
}

impl ManagedNode {
  pub fn create(node_name: impl AsRef<str>) -> Result<Self, Box<dyn std::error::Error>> {
    let mainloop = MainLoopRc::new(None)?;
    let context = ContextRc::new(&mainloop, None)?;
    let core = context.connect_rc(None)?;
    let registry = core.get_registry_rc()?;

    let node = NodeWithPorts::create_virtual_source(&mainloop, &core, node_name)?;

    Ok(Self {
      node_with_ports: node,
      mainloop,
      context,
      core,
      registry,
    })
  }

  pub fn get_node_with_ports(&self) -> &NodeWithPorts {
    &self.node_with_ports
  }
}

impl Drop for ManagedNode {
  fn drop(&mut self) {
    self.registry.destroy_global(self.node_with_ports.id);
    pipewire_utils::do_roundtrip(&self.mainloop, &self.core);
  }
}

pub fn create_virtual_source_if_not_exists(
  mainloop: &MainLoopRc,
  core: &CoreRc,
  node_name: impl AsRef<str> + Copy + 'static,
) -> Option<NodeWithPorts> {
  match pipewire_utils::find_node_by_name(mainloop, core, node_name) {
    Some(node_id) => {
      let ports = pipewire_utils::find_fl_fr_ports(
        node_id,
        pipewire_utils::PortDirection::INPUT,
        mainloop,
        core,
      )
      .expect("node exists but ports don't");
      Some(NodeWithPorts { id: node_id, ports })
    }
    None => NodeWithPorts::create_virtual_source(mainloop, core, &node_name).ok(),
  }
}
