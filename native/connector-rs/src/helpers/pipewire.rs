use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct NodeProperties {
  #[serde(rename = "application.name")]
  application_name: Option<String>,
  #[serde(rename = "media.name")]
  media_name: String,
  #[serde(rename = "object.serial")]
  object_serial: i64,

  #[serde(skip_serializing)]
  #[allow(unused)]
  media_class: String,
}

#[derive(Debug, Serialize)]
pub struct OutputNode {
  id: u32,
  properties: NodeProperties,
}

impl From<pipewire_utils::NodeProperties> for NodeProperties {
  fn from(
    pipewire_utils::NodeProperties {
      application_name,
      media_name,
      object_serial,
      media_class,
    }: pipewire_utils::NodeProperties,
  ) -> Self {
    Self {
      application_name,
      media_name,
      object_serial,
      media_class,
    }
  }
}

impl From<pipewire_utils::OutputNode> for OutputNode {
  fn from(pipewire_utils::OutputNode { id, properties }: pipewire_utils::OutputNode) -> Self {
    Self {
      id,
      properties: properties.into(),
    }
  }
}
