use serde::{Deserialize, Deserializer, Serialize};

use crate::account::Account;

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(rename="type")]
    pub tx_type: TxType,
    pub client: u16,
    pub tx: u32,
    #[serde(deserialize_with="deserialize_amount")]
    pub amount: f32,
    #[serde(default="default_bool", deserialize_with="deserialize_dispute")]
    under_dispute: bool,
}

#[derive(Clone, Debug, Serialize)]
pub enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
    Unknown,
}

impl Transaction {
    pub const DB_NAME: &'static str = "transation_db";

    pub fn deposit(&self, acc: &mut Account) {
        acc.total += self.amount;
        acc.available += self.amount;
    }

    pub fn withdrawal(&self, acc: &mut Account) {
        if self.amount > acc.available {
            return;
        }
        acc.available -= self.amount;
        acc.total -= self.amount;
    }

    pub fn dispute(&mut self, acc: &mut Account) {
        acc.available -= self.amount;
        acc.held += self.amount;
        self.under_dispute = true;
    }

    pub fn resolve(&mut self, acc: &mut Account) {
        acc.available += self.amount;
        acc.held -= self.amount;
        self.under_dispute = false;
    }

    pub fn chargeback(&mut self, acc: &mut Account) {
        acc.total -= self.amount;
        acc.held -= self.amount;
        acc.locked = true;
        self.under_dispute = false;
    }
}

impl<'de> Deserialize<'de> for TxType {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D:Deserializer<'de> {
            let variant = String::deserialize(de)?;
            Ok(match variant.as_str() {
                "deposit" => TxType::Deposit,
                "withdrawal" => TxType::Withdrawal,
                "dispute" => TxType::Dispute,
                "resolve" => TxType::Resolve,
                "chargeback" => TxType::Chargeback,
                _ => TxType::Unknown,
            })
    }
}

fn deserialize_amount<'de, D>(deserializer: D) -> Result<f32, D::Error> 
where D:serde::Deserializer<'de>
{
    let s: Option<f32> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => Ok(s),
        None => Ok(0f32),
    }
}

fn deserialize_dispute<'de, D>(deserializer: D) -> Result<bool, D::Error> 
where D:serde::Deserializer<'de>
{
    let s: Option<bool> = Option::deserialize(deserializer)?;
    match s {
        Some(b) => Ok(b),
        None => Ok(false),
    }
}

fn default_bool() -> bool {
    false
}
