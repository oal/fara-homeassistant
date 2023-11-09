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
struct TravelCard {
    #[serde(rename = "cardNo")]
    card_no: String,
}

#[derive(Deserialize)]
struct TravelCardResponse {
    cards: Vec<TravelCard>,
}

#[derive(Deserialize, Serialize)]
struct TravelProduct {
    status: Option<String>,

    #[serde(rename = "productType")]
    product_type: String,
    #[serde(rename = "templateName")]
    template_name: Option<String>,

    #[serde(rename = "startValidity")]
    start_validity: Option<NaiveDate>,
    #[serde(rename = "endValidity")]
    end_validity: Option<NaiveDate>,

    #[serde(rename = "unitsLeft")]
    units_left: Option<isize>,

    balance: Option<isize>,
}

struct FaraClient {
    pta: String,
    client_id: String,
    client: reqwest::Client,

    token: Option<String>,
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
            // .body(&token_request);
            // .query(&token_request)
            .header("Authorization", format!("Basic {}", authorization))
            .form(&token_request);
        // .build()?;


        // let mut s = String::new();
        // request.body().unwrap().as_bytes().unwrap().read_to_string(&mut s).unwrap();
        // println!("Sending request: {:?}", s);
        // return Ok("".to_string());

        let response = request
            .send()
            .await?;

        let token_response = response.json::<TokenResponse>().await?;
        let token = token_response.access_token.clone();
        self.token = Some(token.clone());
        println!("Got token: {}", token);
        Ok(token)
    }

    pub async fn get_travelcards(&self, username: String) -> anyhow::Result<Vec<TravelCard>> {
        let url = self.url(&format!("users/{username}/travelcards"));

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token.clone().unwrap()))
            .send().await?
            .json::<TravelCardResponse>().await?;

        return Ok(response.cards);
    }

    pub async fn get_travelcard_products(&self, username: String, travelcard_id: String) {
        let url = self.url(&format!("users/{username}/travelcard/{travelcard_id}/products"));

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token.clone().unwrap()))
            .send().await
            .unwrap()
            .json::<Vec<TravelProduct>>().await.unwrap();

        println!("Got travelcard products: {:?}", serde_json::to_string(&response));
    }
}


#[tokio::main]
async fn main() {
    dotenv().expect("Failed to read .env file");
    let username = std::env::var("FARA_USERNAME").expect("FARA_USERNAME not set");
    let password = std::env::var("FARA_PASSWORD").expect("FARA_PASSWORD not set");

    let mut client = FaraClient::new("AKT", "webshop_akt");
    let token = client.get_token(username.clone(), password).await.unwrap();
    let travelcards = client.get_travelcards(username.clone()).await.unwrap();

    for travelcard in travelcards {
        println!("Travelcard: {}", travelcard.card_no);
        client.get_travelcard_products(username.clone(), travelcard.card_no).await;
    }
    // println!("Hello, world! {}", travelcards.len());
}
