use serde::{Serialize, Deserialize};

use crate::account::Account;

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(rename="type")]
    tx_type: TxType,
    client: u32,
    tx: u32,
    amount: f32,
    under_dispute: bool,
}

#[derive(Debug, Serialize, Deserialize)]
enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
    Unknown,
}

impl Transaction {
    pub fn new() -> Transaction {
        Transaction {
            tx_type: TxType::Unknown,
            client: 0,
            tx: 0,
            amount: 0.0,
            under_dispute: false,
        }
    }
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
