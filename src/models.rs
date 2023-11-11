use chrono::NaiveDate;
use crate::homeassistant;
use crate::homeassistant::models::ConfigMessage;

#[derive(Debug, Clone)]
pub struct Card {
    pub id: String,
    pub name: String,
}

#[derive(Debug)]
pub enum Product {
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
        balance: f64,
    },
}

impl Product {
   pub fn name(&self) -> String {
       match self {
           Product::Punch { name, .. } => name.clone(),
           Product::Period { name, .. } => name.clone(),
           Product::Purse { name, .. } => name.clone(),
       }
   }

    pub fn type_name(&self) -> String {
        match self {
            Product::Punch { .. } => "punch".to_string(),
            Product::Period { .. } => "period".to_string(),
            Product::Purse { .. } => "purse".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct CardWithProducts {
    pub card: Card,
    pub products: Vec<Product>,
}

pub struct CardProduct {
    pub(crate) card: Card,
    pub(crate) product: Product,
}

impl CardProduct {
    pub(crate) fn new(card: Card, product: Product) -> Self {
        Self {
            card,
            product,
        }
    }

    pub(crate) fn config_message(&self) -> ConfigMessage {
        homeassistant::helpers::config_message(&self.card, &self.product)
    }

    pub(crate) fn state_message(&self) -> Option<String> {
        homeassistant::helpers::state_message(&self.product)
    }
}
