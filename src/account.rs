use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
}

impl Account {
    pub const DB_NAME: &'static str = "account_db";
    pub fn new(id: u16) -> Account {
        Account {
            id,
            available: 0f32,
            held: 0f32,
            total: 0f32,
            locked: false,
        }
    }
}
