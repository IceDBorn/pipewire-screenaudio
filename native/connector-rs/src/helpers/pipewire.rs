use std::{process::Command, cell::RefCell, thread, time::Duration};

extern crate serde_json;
use serde_json::{json, Deserializer, Map, Value};

extern crate log;
use log::debug;

use crate::helpers::JsonGetters;

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
  let dump: Value = stream.filter(|batch| ! batch.as_ref().unwrap()[0]["info"].is_null()).next().unwrap().unwrap();

  let result = dump.as_array().unwrap().iter().map(|node| node.clone()).collect::<Vec<_>>();

  DUMP_CACHE.with_borrow_mut(|cache| *cache = Some(result.clone()));
  result
}

fn get_node_media_class(node: &Value) -> Result<String,String> {
  let result = node.get_fields_chain(vec!["info","props","media.class"]);
  match result {
    Ok(v) => Ok(v.as_str().unwrap().to_string()),
    Err(e) => Err(e),
  }
}

fn get_node_name(node: &Value) -> Result<String,String> {
  let result = node.get_fields_chain(vec!["info","props","node.name"]);
  match result {
    Ok(v) => Ok(v.as_str().unwrap().to_string()),
    Err(e) => Err(e),
  }
}

pub fn get_output_nodes() -> Vec<Value> {
  let dump = get_pw_dump(false);

  let dump_filtered = dump.iter().filter(|&node| {
    match get_node_media_class(&node) {
      Ok(v) => v == "Stream/Output/Audio",
      Err(e) => {
        debug! ("Error: {}", e);
        return false;
      },
    }
  }).collect::<Vec<_>>();

  let dump_converted = dump_filtered.iter().map(|&node| json!({ "id": node["id"].clone(), "properties": node["info"]["props"].clone() })).collect::<Vec<_>>();

  return dump_converted;
}

pub fn get_node_id_from_serial(serial: i64) -> Option<i64> {
  let dump = get_output_nodes();
  let result = dump.into_iter()
    .find(|node| match node.get_fields_chain(vec!["properties", "object.serial"]) {
      Err(_) => false,
      Ok(v) => v == serial,
    });

  if result.is_some() {
    eprintln!("Found Target: {}", result.to_owned().unwrap());
  }

  match result {
    Some(v) => Some(v["id"].as_i64().unwrap()),
    None => None,
  }
}

pub fn get_ports_of_node(node_id: i64, port_type: String, invalidate_cache: bool) -> Value {
  let ports = get_pw_dump(invalidate_cache).iter()
    .filter(|&node| node["type"] == "PipeWire:Interface:Port")
    .filter(|&node| {
      let is_node_id = match node.get_fields_chain(vec!["info","props","node.id"]) {
        Err(_) => false,
        Ok(v) => {
          eprintln!("{} {}", v, node_id);
          v == node_id
        },
      };

      let is_input = match node.get_fields_chain(vec!["info","direction"]) {
        Err(_) => false,
        Ok(v) => {
          eprintln!("{} {}", v.as_str().unwrap(), port_type);
          v.as_str().unwrap() == port_type
        },
      };

      return is_input && is_node_id;
    })
    .map(|node| json!({
      "id": node["id"].as_i64().unwrap(),
      "channel": node.get_fields_chain(vec!["info","props","audio.channel"]).unwrap(),
    }))
    .collect::<Vec<_>>();

  eprintln!("Ports: {:?}", ports);

  let result = &mut Map::new();
  for port in ports {
    let channel = port["channel"].as_str().unwrap().to_string();
    result.insert(channel, port["id"].clone());
  }

  return Value::Object(result.to_owned());
}

pub fn get_links_to_node(node_id: i64, invalidate_cache: bool) -> Value {
  let links = get_pw_dump(invalidate_cache).iter()
    .filter(|&node| node["type"] == "PipeWire:Interface:Link")
    .filter(|&node| {
      match node.get_fields_chain(vec!["info","input-node-id"]) {
        Err(_) => false,
        Ok(v) => v == node_id,
      }
    })
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

pub fn find_node_by_name(name: &String, invalidate_cache: bool) -> Option<Value> {
  let dump = get_pw_dump(invalidate_cache);

  let found_node = dump.iter().find(|&node| {
    match get_node_name(node) {
      Ok(v) => v == *name,
      Err(_) => false,
    }
  });

  if found_node.is_none() {
    return None;
  }

  Some((*found_node.unwrap()).clone())
}

pub fn node_exists(id: i64, node_name: &String) -> bool { // TODO Result<bool,String>
  let some_node = find_node_by_id(id, false);

  if some_node.is_none() {
    return false;
  }

  match get_node_name(&some_node.unwrap()) {
    Ok(v) => v == *node_name,
    Err(_) => false,
  }
}

pub fn create_virtual_source(node_name: &String) -> i64 { // TODO Result<i64,String>
  let result = Command::new("pw-cli")
    .arg("create-node")
    .arg("adapter")
    .arg(format!("{{ factory.name=support.null-audio-sink node.name={} media.class=Audio/Source/Virtual object.linger=1 audio.position=[FL,FR] }}", node_name))
    .output();

  if result.is_err() {
    return -1;
  }

  thread::sleep(Duration::from_secs(1));

  match find_node_by_name(node_name, true) {
    Some(v) => v["id"].as_i64().unwrap(),
    None => -1,
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
  port_ids.iter()
    .map(|port_id| destroy_node(port_id.clone()))
    .all(|success| success)
}

pub fn connect_ports(port_id_a: i64, port_id_b: i64) -> bool {
  let result = Command::new("pw-link")
    .arg(port_id_a.to_string())
    .arg(port_id_b.to_string())
    .output();

  return result.is_ok();
}

pub fn connect_nodes(in_node_id: i64, out_node_id: i64) -> bool {
  let in_node_ports = get_ports_of_node(in_node_id, "output".to_string(), false);
  eprintln!("Searching in_node ports...");
  if !in_node_ports.as_object().unwrap().contains_key("FL") { return false; }
  if !in_node_ports.as_object().unwrap().contains_key("FR") { return false; }

  let out_node_ports = get_ports_of_node(out_node_id, "input".to_string(), false);
  eprintln!("Searching out_node ports...");
  if !out_node_ports.as_object().unwrap().contains_key("FL") { return false; }
  if !out_node_ports.as_object().unwrap().contains_key("FR") { return false; }

  let result_fl = connect_ports(
    in_node_ports["FL"].as_i64().unwrap(),
    out_node_ports["FL"].as_i64().unwrap(),
  );
  if !result_fl {
    eprintln!("Failed on FL");
    return false;
  }

  let result_fr = connect_ports(
    in_node_ports["FR"].as_i64().unwrap(),
    out_node_ports["FR"].as_i64().unwrap(),
  );
  if !result_fr {
    eprintln!("Failed on FR");
    return false;
  }

  return true;
}

pub fn disconnect_node(node_id: i64) -> bool {
  let links_to_disconnect = get_links_to_node(node_id, false);
  destroy_nodes(links_to_disconnect.as_array().unwrap().iter().map(|link| link["id"].as_i64().unwrap()).collect())
}

pub fn create_virtual_source_if_not_exists(node_name: &String) -> i64 {
  match find_node_by_name(node_name, true) {
    Some(v) => v["id"].as_i64().unwrap(),
    None => create_virtual_source(node_name),
  }
}

pub fn destroy_node_if_exists(node_id: i64) -> bool {
  match find_node_by_id(node_id, false) {
    None => false,
    Some(_) => destroy_node(node_id),
  }
}
