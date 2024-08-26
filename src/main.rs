mod transaction;
mod account;

use std::error::Error;
use std::{fs::File, io::BufReader};

use clap::Parser;
use csv::{ReaderBuilder, Trim};
use sled::Db;
use bincode::{serialize, deserialize};
use transaction::TxType;

use crate::transaction::Transaction;
use crate::account::Account;

#[derive(Parser)]
struct Cli {
    filepath: String,
}

fn main() {
    let cli = Cli::parse();
    let filepath = cli.filepath;
    match process_transactions(filepath) {
        Ok(()) => (),
        Err(e) => eprintln!("Error processing transactions: {}", e),
    };
}

fn process_transactions(filename: String) -> Result<(), Box<dyn Error>>{
    let tx_db = sled::open(Transaction::DB_NAME)?;
    let ac_db = sled::open(Account::DB_NAME)?;
    let file = match File::open(filename) {
        Ok(v) => v,
        _ => panic!("error opening file"),
    };
    let filereader = BufReader::new(file);
    let mut csv_reader = ReaderBuilder::new()
        .trim(Trim::All)
        .has_headers(true)
        .from_reader(filereader);
    for result in csv_reader.deserialize::<Transaction>() {
        let mut tx: Transaction = result?;
        let mut acc: Account = match get_account(&ac_db, tx.client) {
            Ok(a) => match a {
                Some(v) => v,
                None => Account::new(tx.client),
            },
            _ => panic!("Error handling accounts"),
        };
        match tx.tx_type {
            TxType::Deposit => tx.deposit(&mut acc),
            TxType::Withdrawal => tx.withdrawal(&mut acc),
            TxType::Dispute | TxType::Resolve | TxType::Chargeback => {
                let old_type = tx.tx_type.clone();
                tx = match get_transaction(&tx_db, tx.tx) {
                    Ok(Some(v)) => v,
                    _=> continue,
                };
                match old_type {
                    TxType::Resolve => tx.resolve(&mut acc),
                    TxType::Dispute => tx.dispute(&mut acc),
                    TxType::Chargeback => tx.chargeback(&mut acc),
                    _ => unreachable!(),
                }
            },
            TxType::Unknown => continue,
        }
        insert_account(&ac_db, &acc)?;
        insert_transaction(&tx_db, &tx)?;
    }
    Ok(())
}

fn insert_account(db: &Db, account: &Account) -> Result<(), Box<dyn Error>> {
    let serialized_data = serialize(account)?;
    
    db.insert(account.id.to_be_bytes(), serialized_data)?;
    
    db.flush()?;
    
    Ok(())
}

fn get_account(db: &Db, key: u32) -> Result<Option<Account>, Box<dyn Error>> {

    if let Some(serialized_data) = db.get(key.to_be_bytes())? {

        let account: Account = deserialize(&serialized_data)?;
        Ok(Some(account))
    } else {
        Ok(None)
    }
}

fn insert_transaction(db: &Db, tx: &Transaction) -> Result<(), Box<dyn Error>> {
    let serialized_data = serialize(tx)?;
    
    db.insert(tx.tx.to_be_bytes(), serialized_data)?;
    
    db.flush()?;
    
    Ok(())
}

fn get_transaction(db: &Db, key: u32) -> Result<Option<Transaction>, Box<dyn Error>> {

    if let Some(serialized_data) = db.get(key.to_be_bytes())? {

        let tx: Transaction = deserialize(&serialized_data)?;
        Ok(Some(tx))
    } else {
        Ok(None)
    }
}
