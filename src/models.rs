use chrono::NaiveDate;

#[derive(Debug)]
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
        balance: isize,
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
