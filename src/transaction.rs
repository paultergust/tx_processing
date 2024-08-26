use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Transaction {
    #[serde(rename="type")]
    pub tx_type: TxType,
    pub client: u32,
    pub tx: u32,
    pub amount: f32,
}

#[derive(Debug, Serialize, Deserialize)]
enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl Transaction {
    pub fn deposit() {
        todo!("Implement deposit");
    }

    pub fn withdrawal() {
        todo!("Implement deposit");
    }

    pub fn dispute() {
        todo!("Implement deposit");
    }

    pub fn resolve() {
        todo!("Implement deposit");
    }

    pub fn chargeback() {
        todo!("Implement deposit");
    }

}
