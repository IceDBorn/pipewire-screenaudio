use serde::{Deserialize, Serialize};
use std::{
  io::stdout,
  num::ParseIntError,
  os::unix::net::UnixStream,
  sync::atomic::{AtomicBool, Ordering},
};
use thiserror::Error;
use tracing::instrument;

use crate::{
  command::VIRTMIC_NODE_NAME,
  helpers::io::{self},
  ipc,
  monitor::MonitorThreadHandle,
};
use pipewire_utils::{ManagedNode, NodeWithPorts, PipewireClient};

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
  SetInstanceIdentifier { instance_identifier: String },
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
  SetInstanceIdentifierResult,
}

static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

enum SharingNodeState {
  Id(u32),
  AllDesktop,
  NotSharing,
}

struct DaemonState {
  running_thread: Option<MonitorThreadHandle>,
  sharing_node_state: SharingNodeState,
  instance_identifier: Option<String>,
}

impl DaemonState {
  fn reshare(&mut self, virtual_node: &NodeWithPorts, pipewire_client: &PipewireClient) -> bool {
    let _ = self.running_thread.take();
    pipewire_client.unlink_node_ports(virtual_node.ports);
    let success = match self.sharing_node_state {
      SharingNodeState::Id(node_id) => {
        pipewire_client.link_nodes(node_id, virtual_node.ports);
        true
      }
      SharingNodeState::AllDesktop => {
        match MonitorThreadHandle::launch_monitor_thread(
          virtual_node.ports,
          self.instance_identifier.clone(),
        ) {
          Ok(handle) => {
            self.running_thread.replace(handle);
            true
          }
          Err(err) => {
            tracing::error!("error while launching monitor thread: {err}");
            false
          }
        }
      }
      SharingNodeState::NotSharing => true,
    };
    success
  }
}

fn handle_client(
  state: &mut DaemonState,
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
      state.sharing_node_state = match node {
        Some(node_id) => SharingNodeState::Id(node_id),
        None => SharingNodeState::AllDesktop,
      };
      let success = state.reshare(virtual_node, pipewire_client);
      io::write(Response::SetSharingNodeResult { success }, &stream).unwrap();
    }
    Command::SetInstanceIdentifier {
      instance_identifier,
    } => {
      state.instance_identifier = Some(instance_identifier);
      state.reshare(virtual_node, pipewire_client);
      io::write(Response::SetInstanceIdentifierResult, &stream).unwrap();
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

#[instrument]
pub fn monitor_and_connect_nodes() -> Result<(), Error> {
  tracing::info!("starting daemon monitor");

  let pipewire_client = PipewireClient::new().unwrap();

  let pipe = ipc::listen().unwrap();

  let managed_virtual_node =
    ManagedNode::create_managed_node(&pipewire_client, VIRTMIC_NODE_NAME).unwrap();
  let virtual_node = *managed_virtual_node.get_node_with_ports();

  ctrlc::set_handler(|| {
    stop_daemon();
  })
  .unwrap();

  let mut state = DaemonState {
    running_thread: None,
    instance_identifier: None,
    sharing_node_state: SharingNodeState::NotSharing,
  };

  io::write(
    Response::StartResult {
      mic_id: virtual_node.id,
    },
    stdout(),
  )
  .unwrap();

  for stream in pipe.incoming() {
    if !KEEP_RUNNING.load(Ordering::Relaxed) {
      break;
    }
    let Ok(stream) = stream else {
      tracing::warn!("failed to accept incomming connection");
      continue;
    };
    handle_client(&mut state, &pipewire_client, &virtual_node, stream);
  }

  drop(state);

  Ok(())
}
