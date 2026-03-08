#![allow(non_snake_case)]

use std::env;
use std::env::current_exe;
use std::process::Command;
use std::process::Stdio;
use std::str;

use pipewire_utils::PipewireClient;
use serde_json::{json, Value};

use crate::daemon;
use crate::helpers::io;
use crate::helpers::parse_numeric_argument;
use crate::helpers::pipewire::OutputNode;
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
  let client = PipewireClient::new().unwrap();
  let nodes: Vec<_> = client
    .list_output_nodes()
    .into_iter()
    .map(OutputNode::from)
    .collect();
  Ok(serde_json::to_value(nodes).unwrap())
}

fn StartPipewireScreenAudio(_: io::Payload) -> Result<Value, String> {
  let mut daemon_process = Command::new(current_exe().unwrap())
    .arg("daemon")
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .stdin(Stdio::null())
    .spawn()
    .unwrap();

  let daemon_stdout = daemon_process.stdout.take().unwrap();

  let mic_id = ipc_request::read_start_result(daemon_stdout);
  drop(daemon_process);

  Ok(json!({
    "micId": mic_id?
  }))
}

fn SetSharingNode(payload: io::Payload) -> Result<Value, String> {
  let nodes_arg = payload.arguments.get("nodes");

	let mut isAllDesktop = false;

  let nodes = if let Some(nodes_value) = nodes_arg {
    match nodes_value {
      Value::Array(arr) => {
        let mut node_ids = Vec::new();
        let client = PipewireClient::new().unwrap();

        for node_value in arr {
					if node_value == -1 {
						// If any node is -1, treat as AllDesktop (None)
						node_ids.clear();
						isAllDesktop = true;
						break;
					}

          let node_serial = parse_numeric_argument(node_value.clone());
          tracing::debug!("node serial to connect: {node_serial}");

          let Some(node_id) = client.get_node_id_from_object_serial(node_serial) else {
            return Ok(json!({
              "success": false
            }));
          };
          tracing::info!("node id to connect: {node_id}");
          node_ids.push(node_id);
        }

        Some(node_ids)
      }
      _ => {
        return Err("nodes argument must be an array".to_string());
      }
    }
  } else {
    return Err("nodes argument must be an array".to_string());
  };

  let pipe = ipc::connect().map_err(|err| err.to_string())?;
	let nodes = if isAllDesktop { None } else { nodes };
  io::write(daemon::Command::SetSharingNode { nodes }, &pipe).unwrap();
  let res: daemon::Response = io::read(&pipe).unwrap();

  let daemon::Response::SetSharingNodeResult { success } = res else {
    tracing::error!("invalid response for SetSharingNode, {res:?}");
    return Err(format!("invalid response for SetSharingNode, {res:?}"));
  };

  Ok(json!({
    "success": success
  }))
}

fn IsPipewireScreenAudioRunning(_payload: io::Payload) -> Result<Value, String> {
  let is_running = ipc_request::is_daemon_running().is_ok_and(|running| running);
  Ok(json!({
    "isRunning": is_running
  }))
}

fn StopPipewireScreenAudio(_payload: io::Payload) -> Result<Value, String> {
  let success = ipc_request::stop_daemon().is_ok();

  Ok(json!({
    "success": success
  }))
}

fn SetInstanceIdentifier(payload: io::Payload) -> Result<Value, String> {
  let instance_identifier = payload.arguments.get("id").unwrap().as_str().unwrap();
  let success = ipc_request::set_instance_identifier(instance_identifier).is_ok();
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
    "SetInstanceIdentifier" => SetInstanceIdentifier(payload),
    _ => Err(format!("Unknown command: {}", payload.command)),
  }
}
