#![allow(non_snake_case)]

extern crate json;
use json::{object, JsonValue};

use super::io;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn GetVersion(payload: io::Payload) -> Result<JsonValue, String> {
  Ok(object! {
    "version": VERSION
  })
}

fn GetSessionType(payload: io::Payload) -> Result<JsonValue, String> {
  Ok(JsonValue::new_object())
}

fn GetNodes(payload: io::Payload) -> Result<JsonValue, String> {
  Ok(JsonValue::new_object())
}

fn StartPipewireScreenAudio(payload: io::Payload) -> Result<JsonValue, String> {
  Ok(JsonValue::new_object())
}

fn SetSharingNode(payload: io::Payload) -> Result<JsonValue, String> {
  Ok(JsonValue::new_object())
}

fn IsPipewireScreenAudioRunning(payload: io::Payload) -> Result<JsonValue, String> {
  Ok(JsonValue::new_object())
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
