#![allow(non_snake_case)]

use std::env;
use std::str;

extern crate json;
use json::{object, JsonValue};

use crate::helpers::io;
use crate::helpers::pipewire;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const VIRTMIC_NODE_NAME: &str = "pipewire-screenaudio";

fn GetVersion(_: io::Payload) -> Result<JsonValue, String> {
  Ok(object! {
    "version": VERSION
  })
}

fn GetSessionType(_: io::Payload) -> Result<JsonValue, String> {
  let session_type = match env::var_os("WAYLAND_DISPLAY") {
    Some(_) => "wayland",
    None => "x11"
  };

  Ok(object! {
    "type": session_type
  })
}

fn GetNodes(_: io::Payload) -> Result<JsonValue, String> {
  let nodes = pipewire::get_output_nodes();
  Ok(JsonValue::from(nodes))
}

fn StartPipewireScreenAudio(payload: io::Payload) -> Result<JsonValue, String> {
  let node_id = pipewire::create_virtual_source_if_not_exists(&VIRTMIC_NODE_NAME.to_string());

  Ok(object! {
    "micId": node_id
  })
}

fn SetSharingNode(payload: io::Payload) -> Result<JsonValue, String> {
  Ok(JsonValue::new_object())
}

fn IsPipewireScreenAudioRunning(payload: io::Payload) -> Result<JsonValue, String> {
  let is_running = pipewire::node_exists(
    payload.arguments["micId"].as_i32().unwrap(),
    &VIRTMIC_NODE_NAME.to_string(),
  );

  Ok(object! {
    "isRunning": is_running
  })
}

fn StopPipewireScreenAudio(payload: io::Payload) -> Result<JsonValue, String> {
  let node_id = payload.arguments["micId"].as_i32().unwrap();
  let result = pipewire::destroy_node_if_exists(node_id);

  Ok(object! {
    "success": result
  })
}

pub fn run(payload: io::Payload) -> Result<JsonValue, String> {
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
