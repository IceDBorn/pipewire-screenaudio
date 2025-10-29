use crate::{daemon, helpers::io, ipc};

pub fn is_daemon_running() -> Result<bool, String> {
  let pipe = ipc::connect().map_err(|err| err.to_string())?;
  io::write(daemon::Command::Ping, &pipe).map_err(|err| err.to_string())?;
  let res: daemon::Response = io::read(&pipe).map_err(|err| err.to_string())?;

  let daemon::Response::PingResult = res else {
    tracing::error!("invalid response for Ping, {res:?}");
    return Err(format!("invalid response for Ping, {res:?}"));
  };

  Ok(true)
}

pub fn stop_daemon() -> Result<(), String> {
  let pipe = ipc::connect().map_err(|err| err.to_string())?;
  io::write(daemon::Command::Stop, &pipe).map_err(|err| err.to_string())?;
  let res: daemon::Response = io::read(&pipe).map_err(|err| err.to_string())?;

  let daemon::Response::StopResult = res else {
    tracing::error!("invalid response for Stop, {res:?}");
    return Err(format!("invalid response for Stop, {res:?}"));
  };

  Ok(())
}
