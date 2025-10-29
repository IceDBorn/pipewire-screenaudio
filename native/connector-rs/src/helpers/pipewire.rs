use std::{cell::RefCell, process::Command, rc::Rc, thread, time::Duration};

extern crate serde_json;
use pipewire::{
  channel::Receiver, context::ContextRc, core::CoreRc, loop_::Signal, main_loop::MainLoopRc,
  node::NodeInfoRef, properties::PropertiesBox, registry::RegistryRc,
  spa::support::system::IoFlags,
};
use pipewire_utils::Ports;
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

thread_local! {
  static DUMP_CACHE: RefCell<Option<Vec<Value>>> = RefCell::new(None);
}

fn get_pw_dump(invalidate_cache: bool) -> Vec<Value> {
  let dump_cache = DUMP_CACHE.with_borrow(|x| x.clone());
  if !invalidate_cache && dump_cache.is_some() {
    return dump_cache.unwrap();
  }

  let dump_buffer = Command::new("pw-dump")
    .output()
    .expect("failed to execute process")
    .stdout;

  let dump_string = str::from_utf8(&dump_buffer).unwrap();

  // In case `pw-dump` returns multiple arrays (#15), only keep the first one
  let stream = Deserializer::from_str(dump_string).into_iter::<Value>();
  let dump: Value = stream
    .filter(|batch| !batch.as_ref().unwrap()[0]["info"].is_null())
    .next()
    .unwrap()
    .unwrap();

  let result = dump
    .as_array()
    .unwrap()
    .iter()
    .map(|node| node.clone())
    .collect::<Vec<_>>();

  DUMP_CACHE.with_borrow_mut(|cache| *cache = Some(result.clone()));
  result
}

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
  let registry = core.get_registry_rc().map_err(Error::from)?;

  let output_nodes = Rc::new(RefCell::new(vec![]));
  pipewire_utils::iterate_nodes(
    &mainloop,
    &core,
    &registry,
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

pub fn get_ports_of_node(node_id: u32, port_type: String, invalidate_cache: bool) -> Value {
  let ports = get_pw_dump(invalidate_cache)
    .iter()
    .filter(|&node| node["type"] == "PipeWire:Interface:Port")
    .filter(|&node| {
      let is_node_id = match node.get_fields_chain(vec!["info", "props", "node.id"]) {
        Err(_) => false,
        Ok(v) => {
          eprintln!("{} {}", v, node_id);
          v == node_id
        }
      };

      let is_input = match node.get_fields_chain(vec!["info", "direction"]) {
        Err(_) => false,
        Ok(v) => {
          eprintln!("{} {}", v.as_str().unwrap(), port_type);
          v.as_str().unwrap() == port_type
        }
      };

      return is_input && is_node_id;
    })
    .map(|node| {
      json!({
        "id": node["id"].as_i64().unwrap(),
        "channel": node.get_fields_chain(vec!["info","props","audio.channel"]).unwrap(),
      })
    })
    .collect::<Vec<_>>();

  eprintln!("Ports: {:?}", ports);

  let result = &mut Map::new();
  for port in ports {
    let channel = port["channel"].as_str().unwrap().to_string();
    result.insert(channel, port["id"].clone());
  }

  return Value::Object(result.to_owned());
}

pub fn get_links_to_node(node_id: u32, invalidate_cache: bool) -> Value {
  let links = get_pw_dump(invalidate_cache)
    .iter()
    .filter(|&node| node["type"] == "PipeWire:Interface:Link")
    .filter(
      |&node| match node.get_fields_chain(vec!["info", "input-node-id"]) {
        Err(_) => false,
        Ok(v) => v.as_u64().unwrap() as u32 == node_id,
      },
    )
    .map(|link| link.to_owned())
    .collect::<Vec<_>>();

  return Value::Array(links);
}

pub fn find_node_by_id(id: i64, invalidate_cache: bool) -> Option<Value> {
  let dump = get_pw_dump(invalidate_cache);

  let found_node = dump.iter().find(|&node| node["id"].as_i64().unwrap() == id);

  if found_node.is_none() {
    return None;
  }

  Some((*found_node.unwrap()).clone())
}

pub fn find_node_by_name(name: impl AsRef<str>, invalidate_cache: bool) -> Option<Value> {
  let dump = get_pw_dump(invalidate_cache);

  let found_node = dump.iter().find(|&node| match get_node_name(node) {
    Ok(v) => &v == name.as_ref(),
    Err(_) => false,
  });

  if found_node.is_none() {
    return None;
  }

  Some((*found_node.unwrap()).clone())
}

pub fn node_exists(id: i64, node_name: impl AsRef<str>) -> bool {
  // TODO Result<bool,String>
  let some_node = find_node_by_id(id, false);

  if some_node.is_none() {
    return false;
  }

  match get_node_name(&some_node.unwrap()) {
    Ok(v) => &v == node_name.as_ref(),
    Err(_) => false,
  }
}

pub fn destroy_node(node_id: i64) -> bool {
  let result = Command::new("pw-cli")
    .arg("destroy")
    .arg(node_id.to_string())
    .output();

  result.is_ok()
}

pub fn destroy_nodes(port_ids: Vec<i64>) -> bool {
  port_ids
    .iter()
    .map(|port_id| destroy_node(port_id.clone()))
    .all(|success| success)
}

pub fn connect_ports(port_id_a: u32, port_id_b: u32) -> bool {
  let result = Command::new("pw-link")
    .arg(port_id_a.to_string())
    .arg(port_id_b.to_string())
    .output();

  return result.is_ok();
}

pub fn connect_nodes(
  mainloop: &MainLoopRc,
  core: &CoreRc,
  registry: &RegistryRc,
  in_node_id: u32,
  mic_ports: &Ports,
) -> bool {
  pipewire_utils::connect_node(in_node_id, mic_ports, mainloop, core, registry).is_ok()
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
  let registry = core.get_registry_rc()?;
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
        let registry = core.get_registry_rc().unwrap();
        pipewire_utils::connect_node(node, &mic_ports, &mainloop, &core, &registry);
      }
    },
    &mainloop,
    &core,
    &registry,
  );
  log::info!("exited node monitoring loop");
  Ok(())
}

pub fn disconnect_node(node_id: u32) -> bool {
  let links_to_disconnect = get_links_to_node(node_id, false);
  destroy_nodes(
    links_to_disconnect
      .as_array()
      .unwrap()
      .iter()
      .map(|link| link["id"].as_i64().unwrap())
      .collect(),
  )
}

pub struct NodeWithPorts {
  pub id: u32,
  pub ports: Ports,
}

pub fn create_virtual_source(
  mainloop: &MainLoopRc,
  core: &CoreRc,
  registry: &RegistryRc,
  node_name: impl AsRef<str>,
) -> Result<NodeWithPorts, Box<dyn std::error::Error>> {
  let node = pipewire_utils::create_node(&node_name, core).expect("Failed to create node");

  let node_id = pipewire_utils::await_node_creation(node, mainloop, core);
  dbg!(node_id);

  let ports = pipewire_utils::await_find_fl_fr_ports(
    node_id,
    pipewire_utils::PortDirection::INPUT,
    mainloop,
    core,
    registry,
  );

  return Ok(NodeWithPorts { id: node_id, ports });
}

pub fn create_virtual_source_if_not_exists(
  mainloop: &MainLoopRc,
  core: &CoreRc,
  registry: &RegistryRc,
  node_name: impl AsRef<str> + Copy + 'static,
) -> Option<NodeWithPorts> {
  match pipewire_utils::find_node_by_name(mainloop, core, registry, node_name) {
    Some(node_id) => {
      let ports = pipewire_utils::find_fl_fr_ports(
        node_id,
        pipewire_utils::PortDirection::INPUT,
        mainloop,
        core,
        registry,
      )
      .expect("node exists but ports don't");
      Some(NodeWithPorts { id: node_id, ports })
    }
    None => create_virtual_source(mainloop, core, registry, &node_name).ok(),
  }
}

pub fn destroy_node_if_exists(node_id: i64) -> bool {
  match find_node_by_id(node_id, false) {
    None => false,
    Some(_) => destroy_node(node_id),
  }
}
