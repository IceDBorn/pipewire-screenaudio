use serde::{Deserialize, Serialize};
use std::{
  num::ParseIntError,
  os::unix::net::UnixStream,
  sync::atomic::{AtomicBool, Ordering},
};
use thiserror::Error;

use crate::{
  command::VIRTMIC_NODE_NAME,
  helpers::io::{self},
  ipc,
  monitor::MonitorThreadHandle,
};
use pipewire_utils::{utils::ManagedNode, NodeWithPorts, PipewireClient};

#[derive(Error, Debug)]
pub enum ArgParseError {
  #[error("error parsing int")]
  ParseIntError(#[from] ParseIntError),
}

#[derive(Error, Debug)]
pub enum Error {
  #[error("error parsing arguments")]
  ArgParseError(#[from] ArgParseError),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum Command {
  SetSharingNode { node: Option<u32> },
  SetExcludedTitle { title: String },
  Ping,
  Stop,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Response {
  StartResult {
    #[serde(rename = "micId")]
    mic_id: u32,
  },
  SetSharingNodeResult {
    success: bool,
  },
  PingResult,
  StopResult,
}

static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

fn handle_client(
  running_thread: &mut Option<MonitorThreadHandle>,
  pipewire_client: &PipewireClient,
  virtual_node: &NodeWithPorts,
  stream: UnixStream,
) {
  let command = match io::read::<_, Command>(&stream) {
    Ok(command) => command,
    Err(err) => {
      tracing::error!("invalid input from ipc channel: {err:?}");
      return;
    }
  };
  tracing::info!("input: {:?}", command);

  match command {
    Command::SetSharingNode { node } => {
      if let Some(mut running_thread) = running_thread.take() {
        running_thread.stop();
      }
      pipewire_client.unlink_node_ports(virtual_node.ports);
      let success = match node {
        Some(node_id) => {
          pipewire_client.link_nodes(node_id, virtual_node.ports);
          true
        }
        None => match MonitorThreadHandle::launch_monitor_thread(virtual_node.ports) {
          Ok(handle) => {
            running_thread.replace(handle);
            true
          }
          Err(err) => {
            tracing::error!("error while launching monitor thread: {err}");
            false
          }
        },
      };
      io::write(Response::SetSharingNodeResult { success }, &stream).unwrap();
    }
    Command::SetExcludedTitle { title } => {
      todo!()
    }
    Command::Ping => {
      io::write(Response::PingResult, &stream).unwrap();
    }
    Command::Stop => {
      io::write(Response::StopResult, &stream).unwrap();
      stop_daemon();
    }
  }
}

fn stop_daemon() {
  tracing::info!("shutting down");
  KEEP_RUNNING.store(false, Ordering::Relaxed);
  ipc::fake_connect();
}

pub fn monitor_and_connect_nodes() -> Result<(), Error> {
  tracing::info!("starting daemon monitor");

  let pipewire_client = PipewireClient::new().unwrap();

  let pipe = ipc::listen().unwrap();
  let first = pipe.incoming().next().unwrap().unwrap();
  let managed_virtual_node =
    ManagedNode::create_managed_node(&pipewire_client, VIRTMIC_NODE_NAME).unwrap();
  let virtual_node = *managed_virtual_node.get_node_with_ports();

  io::write(
    Response::StartResult {
      mic_id: virtual_node.id,
    },
    &first,
  )
  .unwrap();
  drop(first);

  ctrlc::set_handler(|| {
    stop_daemon();
  })
  .unwrap();

  let mut running_thread: Option<MonitorThreadHandle> = None;

  for stream in pipe.incoming() {
    if !KEEP_RUNNING.load(Ordering::Relaxed) {
      break;
    }
    let Ok(stream) = stream else {
      tracing::warn!("failed to accept incomming connection");
      continue;
    };
    handle_client(&mut running_thread, &pipewire_client, &virtual_node, stream);
  }

  drop(running_thread);

  Ok(())
}
