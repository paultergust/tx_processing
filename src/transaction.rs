use serde::{Deserialize, Deserializer, Serialize};

use crate::account::Account;

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: TxType,
    pub client: u16,
    pub tx: String,
    #[serde(deserialize_with = "deserialize_amount")]
    pub amount: f32,
    #[serde(
        default = "default_bool",
        deserialize_with = "deserialize_dispute",
        serialize_with = "bool_to_string"
    )]
    pub under_dispute: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
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
        if !self.under_dispute {
            println!("{:?}", self);
            return;
        }
        acc.available += self.amount;
        acc.held -= self.amount;
        self.under_dispute = false;
    }

    pub fn chargeback(&mut self, acc: &mut Account) {
        if !self.under_dispute {
            return;
        }
        acc.total -= self.amount;
        acc.held -= self.amount;
        acc.locked = true;
        self.under_dispute = false;
    }
}

impl<'de> Deserialize<'de> for TxType {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let variant = String::deserialize(de)?;
        Ok(match variant.as_str() {
            "Deposit" | "deposit" => TxType::Deposit,
            "Withdrawal" | "withdrawal" => TxType::Withdrawal,
            "Dispute" | "dispute" => TxType::Dispute,
            "Resolve" | "resolve" => TxType::Resolve,
            "Chargeback" | "chargeback" => TxType::Chargeback,
            _ => panic!("Type variant unknown: {:?}", variant.as_str()),
        })
    }
}

fn deserialize_amount<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<f32> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => Ok(s),
        None => Ok(0f32),
    }
}

fn deserialize_dispute<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(b) => Ok(b.parse().map_err(serde::de::Error::custom))?,
        None => Ok(false),
    }
}

fn bool_to_string<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = if *value { "true" } else { "false" };
    serializer.serialize_str(s)
}

fn default_bool() -> bool {
    false
}
