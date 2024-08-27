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

// when amount is missing (disputes, resolves, chargebacks), default value is set to 0.0
fn deserialize_amount<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<f32> = Option::deserialize(deserializer)?;
    match s {
        // set amount value to abs so it won't allow negative operations
        Some(s) => Ok(s.abs()),
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::Account; // Assuming Account is defined in crate::account

    #[test]
    fn test_deposit() {
        let mut account = Account {
            id: 1,
            total: 0.0,
            available: 0.0,
            held: 0.0,
            locked: false,
        };
        let transaction = Transaction {
            tx_type: TxType::Deposit,
            client: 1,
            tx: "1".to_string(),
            amount: 100.0,
            under_dispute: false,
        };

        transaction.deposit(&mut account);
        assert_eq!(account.total, 100.0);
        assert_eq!(account.available, 100.0);
    }

    #[test]
    fn test_withdrawal() {
        let mut account = Account {
            id: 1,
            total: 100.0,
            available: 100.0,
            held: 0.0,
            locked: false,
        };
        let transaction = Transaction {
            tx_type: TxType::Withdrawal,
            client: 1,
            tx: "2".to_string(),
            amount: 50.0,
            under_dispute: false,
        };

        transaction.withdrawal(&mut account);
        assert_eq!(account.total, 50.0);
        assert_eq!(account.available, 50.0);
    }

    #[test]
    fn test_withdrawal_insufficient_funds() {
        let mut account = Account {
            id: 1,
            total: 50.0,
            available: 50.0,
            held: 0.0,
            locked: false,
        };
        let transaction = Transaction {
            tx_type: TxType::Withdrawal,
            client: 1,
            tx: "3".to_string(),
            amount: 100.0,
            under_dispute: false,
        };

        transaction.withdrawal(&mut account);
        assert_eq!(account.total, 50.0); // No change
        assert_eq!(account.available, 50.0); // No change
    }

    #[test]
    fn test_dispute() {
        let mut account = Account {
            id: 1,
            total: 100.0,
            available: 100.0,
            held: 0.0,
            locked: false,
        };
        let mut transaction = Transaction {
            tx_type: TxType::Dispute,
            client: 1,
            tx: "4".to_string(),
            amount: 50.0,
            under_dispute: false,
        };

        transaction.dispute(&mut account);
        assert_eq!(account.available, 50.0);
        assert_eq!(account.held, 50.0);
        assert!(transaction.under_dispute);
    }

    #[test]
    fn test_resolve() {
        let mut account = Account {
            id: 1,
            total: 100.0,
            available: 50.0,
            held: 50.0,
            locked: false,
        };
        let mut transaction = Transaction {
            tx_type: TxType::Resolve,
            client: 1,
            tx: "5".to_string(),
            amount: 50.0,
            under_dispute: true,
        };

        transaction.resolve(&mut account);
        assert_eq!(account.available, 100.0);
        assert_eq!(account.held, 0.0);
        assert!(!transaction.under_dispute);
    }

    #[test]
    fn test_resolve_not_under_dispute() {
        let mut account = Account {
            id: 1,
            total: 100.0,
            available: 100.0,
            held: 0.0,
            locked: false,
        };
        let mut transaction = Transaction {
            tx_type: TxType::Resolve,
            client: 1,
            tx: "6".to_string(),
            amount: 50.0,
            under_dispute: false,
        };

        transaction.resolve(&mut account);
        assert_eq!(account.available, 100.0); // No change
        assert_eq!(account.held, 0.0); // No change
        assert!(!transaction.under_dispute);
    }

    #[test]
    fn test_chargeback() {
        let mut account = Account {
            id: 1,
            total: 100.0,
            available: 50.0,
            held: 50.0,
            locked: false,
        };
        let mut transaction = Transaction {
            tx_type: TxType::Chargeback,
            client: 1,
            tx: "7".to_string(),
            amount: 50.0,
            under_dispute: true,
        };

        transaction.chargeback(&mut account);
        assert_eq!(account.total, 50.0);
        assert_eq!(account.held, 0.0);
        assert!(account.locked);
        assert!(!transaction.under_dispute);
    }

    #[test]
    fn test_chargeback_not_under_dispute() {
        let mut account = Account {
            id: 1,
            total: 100.0,
            available: 100.0,
            held: 0.0,
            locked: false,
        };
        let mut transaction = Transaction {
            tx_type: TxType::Chargeback,
            client: 1,
            tx: "8".to_string(),
            amount: 50.0,
            under_dispute: false,
        };

        transaction.chargeback(&mut account);
        assert_eq!(account.total, 100.0); // No change
        assert_eq!(account.held, 0.0); // No change
        assert!(!account.locked);
        assert!(!transaction.under_dispute);
    }

    #[test]
    fn test_deserialize_amount() {
        use serde_json::json;

        let json_data = json!({
            "type": "deposit",
            "client": 1,
            "tx": "9",
            "amount": -100.0
        });

        let transaction: Transaction = serde_json::from_value(json_data).unwrap();
        assert_eq!(transaction.amount, 100.0); // Absolute value
    }

    #[test]
    fn test_deserialize_dispute() {
        use serde_json::json;

        let json_data = json!({
            "type": "dispute",
            "client": 1,
            "tx": "10",
            "amount": 100.0,
            "under_dispute": "true"
        });

        let transaction: Transaction = serde_json::from_value(json_data).unwrap();
        assert_eq!(transaction.under_dispute, true);
    }

    #[test]
    fn test_bool_to_string() {
        use serde_json::to_string;

        let transaction = Transaction {
            tx_type: TxType::Deposit,
            client: 1,
            tx: "11".to_string(),
            amount: 100.0,
            under_dispute: true,
        };

        let json = to_string(&transaction).unwrap();
        assert!(json.contains("\"under_dispute\":\"true\""));
    }

    #[test]
    fn test_tx_type_deserialize() {
        use serde_json::from_str;

        let json_data = "\"deposit\"";
        let tx_type: TxType = from_str(json_data).unwrap();
        assert_eq!(tx_type, TxType::Deposit);

        let json_data = "\"withdrawal\"";
        let tx_type: TxType = from_str(json_data).unwrap();
        assert_eq!(tx_type, TxType::Withdrawal);

        let json_data = "\"dispute\"";
        let tx_type: TxType = from_str(json_data).unwrap();
        assert_eq!(tx_type, TxType::Dispute);

        let json_data = "\"resolve\"";
        let tx_type: TxType = from_str(json_data).unwrap();
        assert_eq!(tx_type, TxType::Resolve);

        let json_data = "\"chargeback\"";
        let tx_type: TxType = from_str(json_data).unwrap();
        assert_eq!(tx_type, TxType::Chargeback);
    }
}

