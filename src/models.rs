use chrono::NaiveDate;

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

#[derive(Debug)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub products: Vec<Product>,
}
