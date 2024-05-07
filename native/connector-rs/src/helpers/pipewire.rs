use std::{process::Command, rc::Rc, str, thread, time::Duration};

extern crate serde_json;
use serde_json::{json, Deserializer, Map, Value};

extern crate log;
use log::debug;

use crate::helpers::JsonGetters;

static mut DUMP_CACHE: Option<Rc<Vec<Value>>> = None;
fn get_dump_cache() -> Option<&'static Vec<Value>> {
  unsafe {
    match &DUMP_CACHE {
      None => None,
      Some(v) => Some(v),
    }
  }
}

fn get_pw_dump(invalidate_cache: bool) -> &'static [Value] {
  if !invalidate_cache && unsafe { DUMP_CACHE.is_some() } {
    return get_dump_cache().unwrap();
  }

  let dump_buffer = Command::new("pw-dump")
    .output()
    .expect("failed to execute process")
    .stdout;

  let dump_string = str::from_utf8(&dump_buffer).unwrap();

  // In case `pw-dump` returns multiple arrays (#15), only keep the first one
  let mut stream = Deserializer::from_str(dump_string).into_iter::<Value>();
  let dump: Value = stream.filter(|batch| ! batch.as_ref().unwrap()[0]["info"].is_null()).next().unwrap().unwrap();

  let result = dump.as_array().unwrap().iter().map(|node| node.clone()).collect::<Vec<_>>();

  let ptr = Rc::new(result);
  unsafe { DUMP_CACHE = Some(ptr); };

  get_dump_cache().unwrap()
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

  let dump_converted = dump_filtered.iter().map(|&node| json!({ "properties": node["info"]["props"].clone() })).collect::<Vec<_>>();

  return dump_converted;
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
