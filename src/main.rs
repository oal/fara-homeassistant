use std::time::Duration;
use crate::config::AppConfig;
use crate::fara::{FaraClient};
use crate::homeassistant::helpers::config_topic;
use crate::homeassistant::mqtt::HomeAssistantMQTTClient;
use crate::models::{CardProduct, CardWithProducts};

mod config;
mod models;
mod fara;
mod homeassistant;

async fn load_cards_and_products(config: &AppConfig) -> anyhow::Result<Vec<CardWithProducts>> {
    let mut client = FaraClient::new(&config.pta, &config.client_id);
    client.login(config.username.clone(), config.password.clone()).await?;

    let mut travel_cards = client.get_travelcards(config.username.clone()).await?;
    for travel_card in &mut travel_cards {
        let products = client.get_travelcard_products(config.username.clone(), travel_card.card.id.clone()).await;
        travel_card.products = products?;
    };

    Ok(travel_cards)
}

async fn send_to_homeassistant(cards: Vec<CardWithProducts>, config: &AppConfig) -> anyhow::Result<()> {
    let mut mqtt_options = rumqttc::MqttOptions::new("fara-homeassistant", &config.mqtt_host, config.mqtt_port);
    mqtt_options.set_keep_alive(Duration::from_secs(5));

    let mut ha = HomeAssistantMQTTClient::new(mqtt_options);

    for card_with_product in cards {
        for product in card_with_product.products {
            let cp = CardProduct::new(card_with_product.card.clone(), product);

            if let Ok(config_message_json) = serde_json::to_string(&cp.config_message()) {
                ha.publish_and_wait(config_topic(&cp.card, &cp.product), config_message_json).await?;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;

            if let Some(state_message) = cp.state_message() {
                ha.publish_and_wait(homeassistant::helpers::state_topic(&cp.card, &cp.product), state_message).await?;
            }
        }
    }

    let _ = ha.disconnect().await;
    Ok(())
}

#[tokio::main]
async fn main() {
    let config = AppConfig::from_env();
    let cards = load_cards_and_products(&config).await.unwrap();
    let _ = send_to_homeassistant(cards, &config).await;
}
