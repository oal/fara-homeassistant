use serde::Serialize;

#[derive(Serialize)]
pub struct Device {
    pub(crate) identifiers: Vec<&'static str>,
    pub(crate) name: &'static str,
}

#[derive(Serialize)]
pub struct ConfigMessage {
    pub(crate) name: String,
    pub(crate) device_class: Option<String>,
    pub(crate) state_topic: String,
    pub(crate) unique_id: String,
    pub(crate) object_id: String,
    pub(crate) device: Device,
}

#[derive(Serialize)]
pub enum StateMessage {
    Amount(isize),
    Expiration(chrono::DateTime<chrono::Local>)
}