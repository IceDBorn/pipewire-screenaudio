use std::vec::Vec;

extern crate json;
use json::JsonValue;

extern crate log;
use log::debug;

pub mod io;
pub mod pipewire;

pub trait JsonGetters {
  fn get_field_or_fail(&self, field: &str) -> Result<JsonValue,String>;
  fn get_fields_chain(&self, fields: Vec<&str>) -> Result<JsonValue,String>;
}

impl JsonGetters for JsonValue {
  fn get_field_or_fail(&self, field: &str) -> Result<JsonValue,String> {
    if self.has_key(field) && !self[field].is_null() {
      Ok(self[field].clone())
    } else {
      Err(format! ("Field does not exist or null: {}", field))
    }
  }

  fn get_fields_chain(&self, fields: Vec<&str>) -> Result<JsonValue,String> {
    if fields.len() == 0 {
      return Err("The 'fields' vector is empty.".to_string());
    }

    debug! ("Testing field: {}", fields[0]);

    if fields.len() > 1 {
      let value = self.get_field_or_fail(fields[0])?;
      value.get_fields_chain(fields[1..].to_owned())
    } else {
      self.get_field_or_fail(fields[0])
    }
  }
}
