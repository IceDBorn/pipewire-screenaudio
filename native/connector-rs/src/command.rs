#![allow(non_snake_case)]

use std::env;
use std::str;

extern crate serde_json;
use serde_json::{json, Value};

use crate::helpers::io;
use crate::helpers::pipewire;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const VIRTMIC_NODE_NAME: &str = "pipewire-screenaudio";

fn GetVersion(_: io::Payload) -> Result<Value, String> {
  Ok(json!({
    "version": VERSION
  }))
}

fn GetSessionType(_: io::Payload) -> Result<Value, String> {
  let session_type = match env::var_os("WAYLAND_DISPLAY") {
    Some(_) => "wayland",
    None => "x11"
  };

  Ok(json!({
    "type": session_type
  }))
}

fn GetNodes(_: io::Payload) -> Result<Value, String> {
  let nodes = pipewire::get_output_nodes();
  Ok(Value::from(nodes))
}

fn StartPipewireScreenAudio(payload: io::Payload) -> Result<Value, String> {
  let micId = pipewire::create_virtual_source_if_not_exists(&VIRTMIC_NODE_NAME.to_string());

  Ok(json!({
    "micId": micId
  }))
}

fn SetSharingNode(payload: io::Payload) -> Result<Value, String> {
  Ok(json!({}))
}

fn IsPipewireScreenAudioRunning(payload: io::Payload) -> Result<Value, String> {
  let is_running = pipewire::node_exists(
    payload.arguments["micId"].as_number().unwrap().as_i64().unwrap(),
    &VIRTMIC_NODE_NAME.to_string(),
  );

  Ok(json!({
    "isRunning": is_running
  }))
}

fn StopPipewireScreenAudio(payload: io::Payload) -> Result<Value, String> {
  let micId = payload.arguments["micId"].as_number().unwrap().as_i64().unwrap();
  let result = pipewire::destroy_node_if_exists(micId);

  Ok(json!({
    "success": result
  }))
}

pub fn run(payload: io::Payload) -> Result<Value, String> {
  let cmd = payload.command.as_str();
  match cmd {
    "GetVersion" => GetVersion(payload),
    "GetSessionType" => GetSessionType(payload),
    "GetNodes" => GetNodes(payload),
    "StartPipewireScreenAudio" => StartPipewireScreenAudio(payload),
    "SetSharingNode" => SetSharingNode(payload),
    "IsPipewireScreenAudioRunning" => IsPipewireScreenAudioRunning(payload),
    "StopPipewireScreenAudio" => StopPipewireScreenAudio(payload),
    _ => Err(format!("Unknown command: {}", payload.command)),
  }
}
