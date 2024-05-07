use std::str;
use std::io;
use std::io::prelude::{Read,Write};

extern crate serde_json;
use serde_json::{from_str, Value};

extern crate log;
use log::debug;

pub struct Payload {
  pub command: String,
  pub arguments: Value,
}

pub fn read() -> Result<Payload, String> {
  let mut length_buffer = [0u8;4];

  let mut stdin = io::stdin();
  stdin.read_exact(&mut length_buffer).unwrap();

  let length = u32::from_le_bytes(length_buffer);
  debug!("Length: {}", length);

  let mut payload_buffer = vec![0u8; usize::try_from(length).unwrap()];
  stdin.read_exact(&mut payload_buffer).unwrap();

  let payload_string = str::from_utf8(&payload_buffer).unwrap();
  debug!("Payload: {}", payload_string);

  let payload: Value = from_str(payload_string).unwrap();
  let cmd = payload["cmd"].to_string();
  let args = &payload["args"][0];

  debug!("Cmd: {}", cmd);
  debug!("Args: {}", args);

  Ok(Payload {
    command: cmd.to_string(),
    arguments: args.clone(),
  })
}

pub fn write(payload: Value) -> Result<(), String> {
  let payload_string = payload.to_string();
  debug! ("Response: {}", payload_string);

  let payload_buffer = payload_string.as_bytes();
  let length = payload_buffer.len();
  let length_buffer = u32::try_from(length).unwrap().to_le_bytes();

  let mut stdout = io::stdout().lock();
  stdout.write_all(&length_buffer).unwrap();
  stdout.write_all(&payload_buffer).unwrap();

  Ok(())
}
