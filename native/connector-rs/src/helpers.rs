use std::vec::Vec;

extern crate serde_json;
use serde_json::Value;

extern crate log;
use log::debug;

pub mod io;
pub mod pipewire;

pub trait JsonGetters {
  fn get_field_or_fail(&self, field: &str) -> Result<Value,String>;
  fn get_fields_chain(&self, fields: Vec<&str>) -> Result<Value,String>;
}

impl JsonGetters for Value {
  fn get_field_or_fail(&self, field: &str) -> Result<Value,String> {
    if self.as_object().unwrap().contains_key(field) && !self[field].is_null() {
      Ok(self[field].clone())
    } else {
      Err(format! ("Field does not exist or null: {}", field))
    }
  }

  fn get_fields_chain(&self, fields: Vec<&str>) -> Result<Value,String> {
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

pub fn parse_numeric_argument(value: Value) -> i64 {
  if value.is_i64() {
    value.as_number().unwrap().as_i64().unwrap()
  } else {
    value.as_str().unwrap().parse::<i64>().unwrap()
  }
}
