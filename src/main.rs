mod transaction;
mod account;

use std::error::Error;
use std::fs::remove_dir_all;
use std::{fs::File, io::BufReader};

use clap::Parser;
use csv::{ReaderBuilder, Trim, Writer};
use sled::Db;
use serde_json::{from_slice, to_string};
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

fn process_transactions(filename: String) -> Result<(), Box<dyn Error>> {
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
        let tx: Transaction = result?;
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
                match get_transaction(&tx_db, tx.tx) {
                    Ok(Some(mut updated_tx)) => {
                        match tx.tx_type {
                            TxType::Dispute => updated_tx.dispute(&mut acc),
                            TxType::Resolve => updated_tx.resolve(&mut acc),
                            TxType::Chargeback => updated_tx.chargeback(&mut acc),
                            _ => unreachable!(),
                        }
                    },
                    _ => continue,
                }
            },
            TxType::Unknown => continue,
        }
        insert_account(&ac_db, &acc)?;
        insert_transaction(&tx_db, &tx)?;
    }
    let _ = output_db_as_csv(&ac_db);
    let _ = remove_dir_all(Account::DB_NAME);
    let _ = remove_dir_all(Transaction::DB_NAME);
    Ok(())
}

fn insert_account(db: &Db, account: &Account) -> Result<(), Box<dyn Error>> {
    let serialized_data = to_string(account)?;
    
    db.insert(account.id.to_be_bytes(), serialized_data.as_bytes())?;
    
    db.flush()?;
    Ok(())
}

fn get_account(db: &Db, key: u16) -> Result<Option<Account>, Box<dyn Error>> {

    if let Some(serialized_data) = db.get(key.to_be_bytes())? {

        let account: Account = from_slice(&serialized_data)?;
        Ok(Some(account))
    } else {
        Ok(None)
    }
}

fn insert_transaction(db: &Db, tx: &Transaction) -> Result<(), Box<dyn Error>> {
    let serialized_data = to_string(tx)?;
    db.insert(tx.tx.to_be_bytes(), serialized_data.as_bytes())?;
    
    db.flush()?;
    
    Ok(())
}

fn get_transaction(db: &Db, key: u32) -> Result<Option<Transaction>, Box<dyn Error>> {
    if let Some(serialized_data) = db.get(key.to_be_bytes())? {

        let tx: Transaction = from_slice(&serialized_data)?;
        Ok(Some(tx))
    } else {
        Ok(None)
    }
}

fn output_db_as_csv(db: &Db) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_writer(std::io::stdout());

    wtr.write_record(&["client", "available", "held", "total", "locked"])?;

    for result in db.iter() {
        let (_, value) = result?;

        let account: Account = from_slice(&value)?;
        let fmt_av = format!("{:.4}", account.available);
        let fmt_hd = format!("{:.4}", account.held);
        let fmt_tt = format!("{:.4}", account.total);

        wtr.serialize((account.id, fmt_av, fmt_hd, fmt_tt, account.locked))?;
    }

    wtr.flush()?;
    Ok(())
}
