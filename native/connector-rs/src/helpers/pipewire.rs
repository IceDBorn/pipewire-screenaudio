use std::process::Command;
use std::str;

extern crate json;
use json::{object,JsonValue};

extern crate log;
use log::debug;

use crate::helpers::JsonGetters;

fn get_pw_dump() -> Vec<JsonValue> {
  let dump_buffer = Command::new("pw-dump")
    .output()
    .expect("failed to execute process")
    .stdout;

  let dump_string = str::from_utf8(&dump_buffer).unwrap();
  let dump = json::parse(dump_string).unwrap();

  dump.members().map(|node| node.to_owned()).collect::<Vec<_>>()
}

fn get_node_media_class(node: &JsonValue) -> Result<String,String> {
  let result = node.get_fields_chain(vec!["info","props","media.class"]);
  match result {
    Ok(v) => Ok(String::from(v.as_str().unwrap())),
    Err(e) => Err(e),
  }
}

pub fn get_output_nodes() -> Vec<JsonValue> {
  let dump = get_pw_dump();

  let dump_filtered = dump.iter().filter(|&node| {
    match get_node_media_class(&node) {
      Ok(v) => v == "Stream/Output/Audio",
      Err(e) => {
      debug! ("Error: {}", e);
      return false;
      },
    }
  }).collect::<Vec<_>>();

  let dump_converted = dump_filtered.iter().map(|&node| object! { "properties": node["info"]["props"].to_owned() }).collect::<Vec<_>>();

  return dump_converted;
}
