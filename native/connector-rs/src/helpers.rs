use serde_json::Value;

pub mod io;
pub mod pipewire;

pub fn parse_numeric_argument(value: Value) -> i64 {
  if value.is_i64() {
    value.as_number().unwrap().as_i64().unwrap()
  } else {
    value.as_str().unwrap().parse::<i64>().unwrap()
  }
}
