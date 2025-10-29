use std::fmt::Debug;
use std::io;
use std::io::prelude::{Read, Write};
use std::str;

use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};

use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct RawPayload {
  #[serde(rename = "cmd")]
  pub command: String,
  #[serde(rename = "args")]
  pub arguments: [Value; 1],
}

impl From<RawPayload> for Payload {
  fn from(value: RawPayload) -> Self {
    let RawPayload {
      command,
      arguments: [arguments],
    } = value;
    Self { command, arguments }
  }
}

impl From<Payload> for RawPayload {
  fn from(value: Payload) -> Self {
    Self {
      command: value.command,
      arguments: [value.arguments],
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "RawPayload", into = "RawPayload")]
pub struct Payload {
  pub command: String,
  pub arguments: Value,
}

#[derive(Error, Debug)]
pub enum ReadError {
  #[error("error while reading length header")]
  ReadingLengthHeader(std::io::Error),
  #[error("error while reading payload")]
  ReadingPayload(std::io::Error),
  #[error("error while parsing payload")]
  PayloadUTFError(std::str::Utf8Error),
  #[error("error while parsing payload")]
  ParsingPayload(serde_json::Error),
}

#[derive(Error, Debug)]
pub enum WriteError {
  #[error("error while writing length header")]
  WritingLengthHeader(std::io::Error),
  #[error("error while writing payload")]
  WritingPayload(std::io::Error),
  #[error("error while serializing payload")]
  SerializingPayload(serde_json::Error),
}

pub fn read<R: Read, P: for<'a> Deserialize<'a> + Debug>(mut reader: R) -> Result<P, ReadError> {
  let mut length_buffer = [0u8; 4];

  reader
    .read_exact(&mut length_buffer)
    .map_err(ReadError::ReadingLengthHeader)?;

  let length = u32::from_le_bytes(length_buffer);
  log::trace!("Length: {}", length);

  let mut payload_buffer = vec![0u8; length as usize];
  reader
    .read_exact(&mut payload_buffer)
    .map_err(ReadError::ReadingPayload)?;

  let payload_string = str::from_utf8(&payload_buffer).map_err(ReadError::PayloadUTFError)?;

  let payload = serde_json::from_str(payload_string).map_err(ReadError::ParsingPayload)?;
  log::debug!("read payload: {:?}", &payload);

  Ok(payload)
}

pub fn write<W: Write, P: Serialize>(payload: P, mut writer: W) -> Result<(), WriteError> {
  let payload = serde_json::to_string(&payload).map_err(WriteError::SerializingPayload)?;
  log::debug!("writing payload: {}", payload);

  let payload = payload.as_bytes();
  let length = payload.len();
  let length_buffer = u32::try_from(length).unwrap().to_le_bytes();

  writer
    .write_all(&length_buffer)
    .map_err(WriteError::WritingLengthHeader)?;
  writer
    .write_all(&payload)
    .map_err(WriteError::WritingPayload)?;

  Ok(())
}
