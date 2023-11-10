use serde::{Deserialize, Serialize};
use dotenvy::dotenv;
use base64::{Engine as _, engine::general_purpose};
use chrono::NaiveDate;

#[derive(Serialize)]
struct TokenRequest {
    fs_username: String,
    fs_password: String,
    fs_pta: String,
    fs_privacy_policy_accepted: bool,
    grant_type: String,
    client_id: String,
    scope: String,
}

impl TokenRequest {
    fn new(username: String, password: String, pta: String, client_id: String) -> Self {
        TokenRequest {
            fs_username: username,
            fs_password: password,
            fs_pta: pta,
            fs_privacy_policy_accepted: false,
            grant_type: "password".to_string(),
            client_id,
            scope: "read".to_string(),
        }
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: usize,
    scope: String,
}

#[derive(Deserialize)]
struct FaraTravelCard {
    #[serde(rename = "cardNo")]
    card_no: String,
    description: String,
}

#[derive(Deserialize)]
struct TravelCardResponse {
    cards: Vec<FaraTravelCard>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FaraTravelProduct {
    status: Option<String>,

    #[serde(rename = "productType")]
    product_type: String,
    #[serde(rename = "templateName")]
    template_name: Option<String>,

    #[serde(rename = "startValidity")]
    start_validity: Option<NaiveDate>,
    #[serde(rename = "endValidity")]
    end_validity: Option<NaiveDate>,

    #[serde(default, rename = "unitsLeft")]
    units_left: isize,

    #[serde(default)]
    balance: isize,

    currency: Option<String>,
}

struct FaraClient {
    pta: String,
    client_id: String,
    client: reqwest::Client,

    token: Option<String>,
}

#[derive(Debug)]
enum Product {
    Punch {
        name: String,
        units_left: isize,
    },
    Period {
        name: String,
        start: NaiveDate,
        end: NaiveDate,
    },
    Purse {
        name: String,
        balance: isize,
    },
}

#[derive(Debug)]
struct Card {
    id: String,
    name: String,
    products: Vec<Product>,
}

impl Product {
    fn from_fara_product(fara_product: &FaraTravelProduct) -> Option<Self> {
        let name = fara_product.template_name.clone();
        match fara_product.product_type.as_str() {
            "PUNCH" => Some(Product::Punch {
                name: name?,
                units_left: fara_product.units_left,
            }),
            "PERIOD" => Some(Product::Period {
                name: name?,
                start: fara_product.start_validity?,
                end: fara_product.end_validity?,
            }),
            "PURSE" => Some(Product::Purse {
                name: fara_product.currency.clone()?,
                balance: fara_product.balance,
            }),
            _ => None,
        }
    }
}


const TOKEN_URL: &str = "https://oauth.api.fara.no/oauth/token";
const API_BASE_URL: &str = "https://webshop.api.fara.no/api/ptas";


impl FaraClient {
    fn new(pta: &str, client_id: &str) -> Self {
        let mut client = reqwest::ClientBuilder::new()
            .user_agent("Fara Home Assistant 0.1")
            .connect_timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();

        FaraClient {
            pta: pta.to_string(),
            client_id: client_id.to_string(),
            client,
            token: None,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{}/{}", API_BASE_URL, self.pta, path)
    }

    async fn get_token(&mut self, username: String, password: String) -> anyhow::Result<String> {
        if self.token.is_some() {
            return Ok(self.token.clone().unwrap());
        }
        let token_request = TokenRequest::new(username, password, self.pta.clone(), self.client_id.clone());

        let authorization = format!("{}:password", self.client_id);
        let authorization = general_purpose::STANDARD.encode(&authorization);
        let request = self.client.post(TOKEN_URL)
            .header("Authorization", format!("Basic {}", authorization))
            .form(&token_request);

        let response = request
            .send()
            .await?;

        let token_response = response.json::<TokenResponse>().await?;
        let token = token_response.access_token.clone();
        self.token = Some(token.clone());
        Ok(token)
    }

    pub async fn get_travelcards(&self, username: String) -> anyhow::Result<Vec<Card>> {
        let url = self.url(&format!("users/{username}/travelcards"));

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token.clone().unwrap()))
            .send().await?
            .json::<TravelCardResponse>().await?;

        return Ok(response.cards.iter().map(|c| Card {
            id: c.card_no.clone(),
            name: c.description.clone(),
            products: Vec::new(),
        }).collect::<Vec<_>>());
    }

    pub async fn get_travelcard_products(&self, username: String, travelcard_id: String) -> anyhow::Result<Vec<Product>> {
        let url = self.url(&format!("users/{username}/travelcard/{travelcard_id}/products"));

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token.clone().unwrap()))
            .send().await?;

        let response = response
            .json::<Vec<FaraTravelProduct>>().await?;

        Ok(response.iter()
            .filter_map(|p| Product::from_fara_product(p))
            .collect::<Vec<_>>())
    }
}


#[tokio::main]
async fn main() {
    dotenv().expect("Failed to read .env file");
    let username = std::env::var("FARA_USERNAME").expect("FARA_USERNAME not set");
    let password = std::env::var("FARA_PASSWORD").expect("FARA_PASSWORD not set");
    let pta = std::env::var("FARA_PTA").expect("FARA_PTA not set");
    let client_id = std::env::var("FARA_CLIENT_ID").expect("FARA_CLIENT_ID not set");

    let mut client = FaraClient::new(&pta, &client_id);
    client.get_token(username.clone(), password).await.unwrap();

    let mut travel_cards = client.get_travelcards(username.clone()).await.unwrap();
    for travel_card in &mut travel_cards {
        let products = client.get_travelcard_products(username.clone(), travel_card.id.clone()).await;
        travel_card.products = products.unwrap();
    };

    println!("Cards: {:?}", travel_cards);
}
