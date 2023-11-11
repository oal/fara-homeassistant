use dotenvy::dotenv;
use crate::fara::{FaraClient};
use crate::models::Card;

mod models;
mod fara;

struct AppConfig {
    username: String,
    password: String,
    pta: String,
    client_id: String,
}

impl AppConfig {
    fn from_env() -> Self {
        dotenv().expect("Failed to read .env file");
        let username = std::env::var("FARA_USERNAME").expect("FARA_USERNAME not set");
        let password = std::env::var("FARA_PASSWORD").expect("FARA_PASSWORD not set");
        let pta = std::env::var("FARA_PTA").expect("FARA_PTA not set");
        let client_id = std::env::var("FARA_CLIENT_ID").expect("FARA_CLIENT_ID not set");

        Self {
            username,
            password,
            pta,
            client_id,
        }
    }
}

async fn load_cards_and_products(config: &AppConfig) -> anyhow::Result<Vec<Card>> {
    let mut client = FaraClient::new(&config.pta, &config.client_id);
    client.login(config.username.clone(), config.password.clone()).await.unwrap();

    let mut travel_cards = client.get_travelcards(config.username.clone()).await.unwrap();
    for travel_card in &mut travel_cards {
        let products = client.get_travelcard_products(config.username.clone(), travel_card.id.clone()).await;
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
}
