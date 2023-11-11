use base64::{Engine as _, engine::general_purpose};
use crate::fara::models::{FaraTravelProduct, TokenRequest, TokenResponse, TravelCardResponse};
use crate::models::{Card, CardWithProducts, Product};

mod models;

const TOKEN_URL: &str = "https://oauth.api.fara.no/oauth/token";
const API_BASE_URL: &str = "https://webshop.api.fara.no/api/ptas";

pub struct FaraClient {
    pta: String,
    client_id: String,
    client: reqwest::Client,

    token: Option<String>,
}

impl FaraClient {
    pub(crate) fn new(pta: &str, client_id: &str) -> Self {
        let client = reqwest::ClientBuilder::new()
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

    pub async fn login(&mut self, username: String, password: String) -> anyhow::Result<String> {
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

    pub async fn get_travelcards(&self, username: String) -> anyhow::Result<Vec<CardWithProducts>> {
        let url = self.url(&format!("users/{username}/travelcards"));

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token.clone().unwrap()))
            .send().await?
            .json::<TravelCardResponse>().await?;

        return Ok(response.cards.iter().map(|c| CardWithProducts {
            card: Card {
                id: c.card_no.clone(),
                name: c.description.clone(),
            },
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
            .filter_map(|p| p.to_product())
            .collect::<Vec<_>>())
    }
}
