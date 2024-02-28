#![allow(non_snake_case)]

use std::env;
use std::process::Command;
use std::str;

extern crate json;
use json::{object, JsonValue};

extern crate log;
use log::debug;

use super::io;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn GetVersion(payload: io::Payload) -> Result<JsonValue, String> {
  Ok(object! {
    "version": VERSION
  })
}

fn GetSessionType(payload: io::Payload) -> Result<JsonValue, String> {
  let session_type = match env::var_os("WAYLAND_DISPLAY") {
    Some(val) => "wayland",
    None => "x11"
  };

  Ok(object! {
    "type": session_type
  })
}

fn GetNodes(payload: io::Payload) -> Result<JsonValue, String> {
  let dump_buffer = Command::new("pw-dump")
    .output()
    .expect("failed to execute process")
    .stdout;

  let dump_string = str::from_utf8(&dump_buffer).unwrap();
  let dump = json::parse(dump_string).unwrap();

  let dump_filtered = dump.members().filter(|&node| {
    if !node.has_key("info") { return false; }
    let info = node["info"].to_owned();
    debug! ("Info: {}", info.dump());
    if info.is_null() { return false; }

    if !info.has_key("props") { return false; }
    let props = info["props"].to_owned();
    debug! ("Props: {}", props.dump());
    if props.is_null() { return false; }

    if !props.has_key("media.class") { return false; }
    let media_class = props["media.class"].to_owned();
    debug! ("Media Class: {}", media_class.dump());
    if media_class.is_null() { return false; }

    let media_class_string = media_class.as_str().unwrap();
    media_class_string == "Stream/Output/Audio"
  }).collect::<Vec<_>>();

  let dump_converted = dump_filtered.iter().map(|&node| object! { "properties": node["info"]["props"].to_owned() }).collect::<Vec<_>>();

  Ok(JsonValue::from(dump_converted))
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
