use std::process::ChildStdout;

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

pub fn set_instance_identifier(instance_identifier: &str) -> Result<(), String> {
  let pipe = ipc::connect().map_err(|err| err.to_string())?;
  io::write(
    daemon::Command::SetInstanceIdentifier {
      instance_identifier: instance_identifier.to_owned(),
    },
    &pipe,
  )
  .map_err(|err| err.to_string())?;
  let res: daemon::Response = io::read(&pipe).map_err(|err| err.to_string())?;

  let daemon::Response::SetInstanceIdentifierResult = res else {
    tracing::error!("invalid response for SetExcludedTitle, {res:?}");
    return Err(format!("invalid response for SetExcludedTitle, {res:?}"));
  };

  Ok(())
}

pub fn read_start_result(daemon_stdout: ChildStdout) -> Result<u32, String> {
  let status: daemon::Response = io::read(daemon_stdout)
    .map_err(|err| format!("error obtaining first response from daemon: {err}"))?;

  let daemon::Response::StartResult { mic_id } = status else {
    return Err(format!(
      "first response from daemon has unexpected format: {status:?}"
    ));
  };
  Ok(mic_id)
}
