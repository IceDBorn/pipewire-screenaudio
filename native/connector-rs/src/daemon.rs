use ::pipewire::{context::ContextRc, core::CoreRc, main_loop::MainLoopRc};
use pipewire_utils::Ports;
use serde::{Deserialize, Serialize};
use std::{
  cell::{Cell, RefCell},
  env::Args,
  num::ParseIntError,
  os::unix::net::UnixStream,
  sync::atomic::{AtomicBool, Ordering},
};
use thiserror::Error;

use crate::{
  command::VIRTMIC_NODE_NAME,
  helpers::{
    io::{self, Payload},
    pipewire::{self, NodeWithPorts},
  },
  ipc,
  monitor::MonitorThreadHandle,
};

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

struct MonitorArgs {
  excluded_nodes: Vec<u32>,
}

static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

fn handle_client(
  running_thread: &mut Option<MonitorThreadHandle>,
  mainloop: &MainLoopRc,
  context: &ContextRc,
  core: &CoreRc,
  virtual_node: &NodeWithPorts,
  stream: UnixStream,
) {
  let command = match io::read::<_, Command>(&stream) {
    Ok(command) => command,
    Err(err) => {
      log::error!("invalid input from ipc channel: {err:?}");
      return;
    }
  };
  log::info!("input: {:?}", command);

  match command {
    Command::SetSharingNode { node } => {
      if let Some(mut running_thread) = running_thread.take() {
        running_thread.stop();
      }
      pipewire::disconnect_node(&mainloop, &core, &virtual_node.ports);
      let success = match node {
        Some(node_id) => pipewire::connect_nodes(&mainloop, &core, node_id, &virtual_node.ports),
        None => match MonitorThreadHandle::launch_monitor_thread(virtual_node.ports) {
          Ok(handle) => {
            running_thread.replace(handle);
            true
          }
          Err(err) => {
            log::error!("error while launching monitor thread: {err}");
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
  log::info!("shutting down");
  KEEP_RUNNING.store(false, Ordering::Relaxed);
  ipc::fake_connect();
}

pub fn monitor_and_connect_nodes() -> Result<(), Error> {
  log::info!("starting daemon monitor");

  let mainloop = MainLoopRc::new(None).unwrap();
  let context = ContextRc::new(&mainloop, None).unwrap();
  let core = context.connect_rc(None).unwrap();

  let pipe = ipc::listen().unwrap();
  let first = pipe.incoming().next().unwrap().unwrap();
  let managed_virtual_node = pipewire::ManagedNode::create(VIRTMIC_NODE_NAME).unwrap();
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
      log::warn!("failed to accept incomming connection");
      continue;
    };
    handle_client(
      &mut running_thread,
      &mainloop,
      &context,
      &core,
      &virtual_node,
      stream,
    );
  }

  if let Some(mut running_thread) = running_thread.take() {
    running_thread.stop();
  }

  Ok(())
}
