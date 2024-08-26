use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Account {
    id: u32,
    available: f32,
    held: f32,
    total: f32,
    locked: bool,
}

impl Account {
    pub fn new(id: u32) -> Account {
        Account {
            id,
            available: 0f32,
            held: 0f32,
            total: 0f32,
            locked: false,
        }
    }
}
