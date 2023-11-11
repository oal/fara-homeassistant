use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use crate::models::Product;

#[derive(Serialize)]
pub struct TokenRequest {
    fs_username: String,
    fs_password: String,
    fs_pta: String,
    fs_privacy_policy_accepted: bool,
    grant_type: String,
    client_id: String,
    scope: String,
}

impl TokenRequest {
    pub(crate) fn new(username: String, password: String, pta: String, client_id: String) -> Self {
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
pub struct TokenResponse {
    pub(crate) access_token: String,
    token_type: String,
    expires_in: usize,
    scope: String,
}

#[derive(Deserialize)]
pub struct FaraTravelCard {
    #[serde(rename = "cardNo")]
    pub(crate) card_no: String,
    pub(crate) description: String,
}

#[derive(Deserialize)]
pub struct TravelCardResponse {
    pub(crate) cards: Vec<FaraTravelCard>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FaraTravelProduct {
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

impl FaraTravelProduct {
    pub(crate) fn to_product(&self) -> Option<Product> {
        let name = self.template_name.clone();
        match self.product_type.as_str() {
            "PUNCH" => Some(Product::Punch {
                name: name?,
                units_left: self.units_left,
            }),
            "PERIOD" => Some(Product::Period {
                name: name?,
                start: self.start_validity?,
                end: self.end_validity?,
            }),
            "PURSE" => Some(Product::Purse {
                name: self.currency.clone()?,
                balance: self.balance,
            }),
            _ => None,
        }
    }
}
