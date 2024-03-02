use std::{process::Command, rc::Rc, str};

extern crate json;
use json::{object,JsonValue};

extern crate log;
use log::debug;

use crate::helpers::JsonGetters;

static mut DUMP_CACHE: Option<Rc<Vec<JsonValue>>> = None;
fn get_dump_cache() -> Option<&'static Vec<JsonValue>> {
  unsafe {
    match &DUMP_CACHE {
      None => None,
      Some(v) => Some(v),
    }
  }
}

fn get_pw_dump(invalidate_cache: bool) -> &'static [JsonValue] {
  if !invalidate_cache && unsafe { DUMP_CACHE.is_some() } {
    return get_dump_cache().unwrap();
  }

  let dump_buffer = Command::new("pw-dump")
    .output()
    .expect("failed to execute process")
    .stdout;

  let dump_string = str::from_utf8(&dump_buffer).unwrap();
  let dump = json::parse(dump_string).unwrap();

  let result = dump.members().map(|node| node.clone()).collect::<Vec<_>>();

  let ptr = Rc::new(result);
  unsafe { DUMP_CACHE = Some(ptr); };

  get_dump_cache().unwrap()
}

fn get_node_media_class(node: &JsonValue) -> Result<String,String> {
  let result = node.get_fields_chain(vec!["info","props","media.class"]);
  match result {
    Ok(v) => Ok(v.to_string()),
    Err(e) => Err(e),
  }
}

fn get_node_name(node: &JsonValue) -> Result<String,String> {
  let result = node.get_fields_chain(vec!["info","props","node.name"]);
  match result {
    Ok(v) => Ok(v.to_string()),
    Err(e) => Err(e),
  }
}

pub fn get_output_nodes() -> Vec<JsonValue> {
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

  let dump_converted = dump_filtered.iter().map(|&node| object! { "properties": node["info"]["props"].clone() }).collect::<Vec<_>>();

  return dump_converted;
}

pub fn find_node_by_id(id: i32) -> Option<JsonValue> {
  let dump = get_pw_dump(false);

  let found_node = dump.iter().find(|&node| node["id"].as_i32().unwrap() == id);

  if found_node.is_none() {
    return None;
  }

  Some((*found_node.unwrap()).clone())
}

pub fn find_node_by_name(name: &String) -> Option<JsonValue> {
  let dump = get_pw_dump(false);

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

pub fn node_exists(id: i32, node_name: &String) -> bool { // TODO Result<bool,String>
  let some_node = find_node_by_id(id);

  if some_node.is_none() {
    return false;
  }

  match get_node_name(&some_node.unwrap()) {
    Ok(v) => v == *node_name,
    Err(_) => false,
  }
}

pub fn create_virtual_source(node_name: &String) -> i32 { // TODO Result<i32,String>
  let result = Command::new("pw-cli")
    .arg("create-node")
    .arg("adapter")
    .arg(format!("{{ factory.name=support.null-audio-sink node.name={} media.class=Audio/Source/Virtual object.linger=1 audio.position=[FL,FR] }}", node_name))
    .output();

  if result.is_err() {
    return -1;
  }

  match find_node_by_name(node_name) {
    Some(v) => v["id"].as_i32().unwrap(),
    None => -1,
  }
}

pub fn create_virtual_source_if_not_exists(node_name: &String) -> i32 {
  match find_node_by_name(node_name) {
    Some(v) => v["id"].as_i32().unwrap(),
    None => create_virtual_source(node_name),
  }
}
