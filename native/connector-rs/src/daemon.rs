use ::pipewire::{context::ContextRc, main_loop::MainLoopRc};
use pipewire_utils::Ports;
use serde::{Deserialize, Serialize};
use std::{env::Args, num::ParseIntError};
use thiserror::Error;

use crate::{
  command::VIRTMIC_NODE_NAME,
  helpers::{
    io::{self, Payload},
    pipewire,
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
}

struct MonitorArgs {
  excluded_nodes: Vec<u32>,
}

fn parse_args(mut args: Args) -> Result<MonitorArgs, ArgParseError> {
  let excluded_nodes: Vec<u32> = args
    .next()
    .iter()
    .map(|arg| arg.parse())
    .collect::<Result<Vec<_>, _>>()
    .map_err(ArgParseError::ParseIntError)?;
  Ok(MonitorArgs { excluded_nodes })
}

pub fn monitor_and_connect_nodes() -> Result<(), Error> {
  log::info!("starting daemon monitor");

  let mainloop = MainLoopRc::new(None).unwrap();
  let context = ContextRc::new(&mainloop, None).unwrap();
  let core = context.connect_rc(None).unwrap();
  let registry = core.get_registry_rc().unwrap();

  let pipe = ipc::listen().unwrap();
  let first = pipe.incoming().next().unwrap().unwrap();
  let mic_id =
    pipewire::create_virtual_source(&mainloop, &core, &registry, VIRTMIC_NODE_NAME).unwrap();

  io::write(Response::StartResult { mic_id: mic_id.id }, &first).unwrap();
  drop(first);

  let mut running_thread: Option<MonitorThreadHandle> = None;

  for stream in pipe.incoming() {
    let Ok(stream) = stream else {
      log::warn!("failed to accept incomming connection");
      continue;
    };
    let command = match io::read::<_, Command>(&stream) {
      Ok(command) => command,
      Err(err) => {
        log::error!("invalid input from ipc channel: {err:?}");
        continue;
      }
    };
    log::info!("input: {:?}", command);

    match command {
      Command::SetSharingNode { node } => {
        if let Some(running_thread) = running_thread.take() {
          running_thread.stop();
        }
        pipewire::disconnect_node(mic_id.id);
        let success = match node {
          Some(node_id) => {
            pipewire::connect_nodes(&mainloop, &core, &registry, node_id, &mic_id.ports)
          }
          None => match MonitorThreadHandle::launch_monitor_thread(mic_id.ports) {
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
    }
  }

  Ok(())
}
