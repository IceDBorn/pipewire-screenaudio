use crate::{daemon, helpers::io, ipc};

pub fn is_daemon_running() -> Result<bool, String> {
  let pipe = ipc::connect().map_err(|err| err.to_string())?;
  io::write(daemon::Command::Ping, &pipe).map_err(|err| err.to_string())?;
  let res: daemon::Response = io::read(&pipe).map_err(|err| err.to_string())?;

  let daemon::Response::PingResult = res else {
    log::error!("invalid response for SetSharingNode, {res:?}");
    return Err(format!("invalid response for SetSharingNode, {res:?}"));
  };

  Ok(true)
}
