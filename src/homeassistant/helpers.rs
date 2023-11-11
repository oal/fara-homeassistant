use chrono::{NaiveTime, TimeZone};
use crate::homeassistant::{INTEGRATION_IDENTIFIER, INTEGRATION_NAME};
use crate::homeassistant::models::{ConfigMessage, Device, StateMessage};
use crate::models::{Card, Product};

pub(crate) fn identifier(card: &Card, product: &Product) -> String {
    format!("{}_{}_{}", INTEGRATION_IDENTIFIER, card.id, product.type_name())
}

pub(crate) fn topic_prefix(card: &Card, product: &Product) -> String {
    let identifier = identifier(card, product);
    format!("homeassistant/sensor/{}", identifier)
}

pub(crate) fn config_topic(card: &Card, product: &Product) -> String {
    let prefix = topic_prefix(card, product);
    format!("{}/config", prefix)
}

pub(crate) fn state_topic(card: &Card, product: &Product) -> String {
    let prefix = topic_prefix(card, product);
    format!("{}/state", prefix)
}

fn device_class(product: &Product) -> Option<String> {
    match product {
        Product::Punch { .. } => None,
        Product::Period { .. } => Some("timestamp".to_string()),
        Product::Purse { .. } => Some("monetary".to_string()),
    }
}

pub fn config_message(card: &Card, product: &Product) -> ConfigMessage {
    let identifier = identifier(card, product);
    ConfigMessage {
        name: format!("{}: {}", card.name, product.name()),
        device_class: device_class(product),
        state_topic: state_topic(card, product),
        unique_id: identifier.clone(),
        object_id: identifier,
        device: Device {
            identifiers: vec![INTEGRATION_IDENTIFIER],
            name: INTEGRATION_NAME,
        },
    }
}

pub fn state_message(product: &Product) -> Option<String> {
    match product {
        Product::Punch { units_left, .. } => Some(units_left.to_string()),
        Product::Period { end, .. } => {
            let date_time = end.and_time(NaiveTime::MIN);
            let local_date_time = chrono::Local.from_local_datetime(&date_time).single()?;
            Some(local_date_time.format("%+").to_string())
        }
        Product::Purse { balance, .. } => Some(balance.to_string()),
    }
}
