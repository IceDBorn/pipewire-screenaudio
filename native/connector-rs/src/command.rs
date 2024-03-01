#![allow(non_snake_case)]

use std::env;
use std::str;

extern crate json;
use json::{object, JsonValue};

use crate::helpers::io;
use crate::helpers::pipewire;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
  Ok(JsonValue::new_object())
}

fn SetSharingNode(payload: io::Payload) -> Result<JsonValue, String> {
  Ok(JsonValue::new_object())
}

fn IsPipewireScreenAudioRunning(payload: io::Payload) -> Result<JsonValue, String> {
  let is_running = pipewire::node_exists(
    payload.arguments["id"].as_i32().unwrap(),
    "pipewire-screenaudio".to_string(),
  );

  Ok(object! {
    "isRunning": is_running
  })
}

fn StopPipewireScreenAudio(payload: io::Payload) -> Result<JsonValue, String> {
  Ok(JsonValue::new_object())
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
