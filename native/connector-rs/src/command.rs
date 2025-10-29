#![allow(non_snake_case)]

use std::env;
use std::env::current_exe;
use std::panic;
use std::process;
use std::process::Command;
use std::str;
use std::thread;
use std::time::Duration;

extern crate serde_json;
use serde_json::{json, Value};

use crate::daemon;
use crate::helpers::io;
use crate::helpers::io::Payload;
use crate::helpers::parse_numeric_argument;
use crate::helpers::pipewire;
use crate::ipc;
use crate::ipc_request;

const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const VIRTMIC_NODE_NAME: &str = "pipewire-screenaudio";

fn GetVersion(_: io::Payload) -> Result<Value, String> {
  Ok(json!({
    "version": VERSION
  }))
}

fn GetSessionType(_: io::Payload) -> Result<Value, String> {
  let session_type = match env::var_os("WAYLAND_DISPLAY") {
    Some(_) => "wayland",
    None => "x11",
  };

  Ok(json!({
    "type": session_type
  }))
}

fn GetNodes(_: io::Payload) -> Result<Value, String> {
  let nodes = pipewire::get_output_nodes().unwrap();
  Ok(serde_json::to_value(nodes).unwrap())
}

fn StartPipewireScreenAudio(payload: io::Payload) -> Result<Value, String> {
  Command::new(current_exe().unwrap())
    .arg("daemon")
    .spawn()
    .unwrap();

  let pipe = ipc::connect().map_err(|err| err.to_string())?;
  let status: daemon::Response =
    io::read(pipe).map_err(|err| format!("error obtaining first response from daemon: {err}"))?;
  let daemon::Response::StartResult { mic_id } = status else {
    return Err("first response from daemon has unexpected format".into());
  };

  Ok(json!({
    "micId": mic_id
  }))
}

fn SetSharingNode(payload: io::Payload) -> Result<Value, String> {
  let node = parse_numeric_argument(payload.arguments["node"].clone());

  log::debug!("node serial to connect: {node}");
  let node = if node == -1 {
    None
  } else {
    let Some(node) = pipewire::get_node_id_from_serial(node) else {
      return Ok(json!({
        "success": false
      }));
    };
    log::debug!("node id to connect: {node}");
    Some(node)
  };

  let pipe = ipc::connect().map_err(|err| err.to_string())?;
  io::write(daemon::Command::SetSharingNode { node }, &pipe).unwrap();
  let res: daemon::Response = io::read(&pipe).unwrap();

  let daemon::Response::SetSharingNodeResult { success } = res else {
    log::error!("invalid response for SetSharingNode, {res:?}");
    return Err(format!("invalid response for SetSharingNode, {res:?}"));
  };

  Ok(json!({
    "success": success
  }))
}

fn IsPipewireScreenAudioRunning(payload: io::Payload) -> Result<Value, String> {
  let is_running = ipc_request::is_daemon_running().is_ok_and(|running| running);
  Ok(json!({
    "isRunning": is_running
  }))
}

fn StopPipewireScreenAudio(payload: io::Payload) -> Result<Value, String> {
  let success = ipc_request::stop_daemon().is_ok();

  Ok(json!({
    "success": success
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
