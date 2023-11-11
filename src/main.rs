use std::time::Duration;
use dotenvy::dotenv;
use rumqttc::{AsyncClient, QoS};
use crate::fara::{FaraClient};
use crate::homeassistant::helpers::config_topic;
use crate::models::{CardWithProducts};

mod models;
mod fara;
mod homeassistant;

struct AppConfig {
    username: String,
    password: String,
    pta: String,
    client_id: String,

    mqtt_host: String,
    mqtt_port: u16,
}

impl AppConfig {
    fn from_env() -> Self {
        dotenv().expect("Failed to read .env file");
        use std::env::var;
        let port = var("MQTT_PORT").or::<()>(Ok("1883".to_string())).unwrap();
        let mqtt_port = port.parse::<u16>().expect("MQTT_PORT must be a valid port number.");
        Self {
            username: var("FARA_USERNAME").expect("FARA_USERNAME not set"),
            password: var("FARA_PASSWORD").expect("FARA_PASSWORD not set"),
            pta: var("FARA_PTA").expect("FARA_PTA not set"),
            client_id: var("FARA_CLIENT_ID").expect("FARA_CLIENT_ID not set"),

            mqtt_host: var("MQTT_HOST").expect("MQTT_HOST not set"),
            mqtt_port,
        }
    }
}

async fn load_cards_and_products(config: &AppConfig) -> anyhow::Result<Vec<CardWithProducts>> {
    let mut client = FaraClient::new(&config.pta, &config.client_id);
    client.login(config.username.clone(), config.password.clone()).await.unwrap();

    let mut travel_cards = client.get_travelcards(config.username.clone()).await.unwrap();
    for travel_card in &mut travel_cards {
        let products = client.get_travelcard_products(config.username.clone(), travel_card.card.id.clone()).await;
        travel_card.products = products.unwrap();
    };

    Ok(travel_cards)
}

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to read .env file");

    let config = AppConfig::from_env();
    let cards = load_cards_and_products(&config).await.unwrap();
    println!("Cards: {:?}", cards);

    let mut mqttoptions = rumqttc::MqttOptions::new("fara-homeassistant", config.mqtt_host, config.mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut mqtt_client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    for card in &cards {
        for product in &card.products {
            let config_message = homeassistant::helpers::config_message(&card.card, product);
            let config_message_json = serde_json::to_string(&config_message).unwrap();
            println!("Config message: {}", config_message_json);
            // let state_message = homeassistant::helpers::state_message(product).unwrap();
            mqtt_client.publish(config_topic(&card.card, product), QoS::AtLeastOnce, false, config_message_json).await.unwrap();
            let notification = eventloop.poll().await.unwrap();
            let notification = eventloop.poll().await.unwrap();

            let state_message = homeassistant::helpers::state_message(product).unwrap();
            mqtt_client.publish(homeassistant::helpers::state_topic(&card.card, product), QoS::AtLeastOnce, false, state_message).await.unwrap();
            let notification = eventloop.poll().await.unwrap();
            let notification = eventloop.poll().await.unwrap();
        }
    }

    mqtt_client.disconnect().await.unwrap();
}
