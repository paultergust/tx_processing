mod account;
mod transaction;

use std::error::Error;
use std::fs::{remove_dir_all, File};
use std::io::BufReader;

use clap::Parser;
use csv::{ReaderBuilder, Trim, Writer};
use serde_json::{from_slice, to_string};
use sled::Db;
use transaction::TxType;

use crate::account::Account;
use crate::transaction::Transaction;

#[derive(Parser)]
struct Cli {
    filepath: String,
}

fn main() {
    let cli = Cli::parse();
    let filepath = cli.filepath;

    if let Err(e) = process_transactions(filepath) {
        eprintln!("Error processing transactions: {}", e);
    }
}

fn process_transactions(filename: String) -> Result<(), Box<dyn Error>> {
    // two different K/V databases, to hold Accounts and Transactions on disk instead of in memory,
    // in a somewhat "hashmap" fashion
    let tx_db = sled::open(Transaction::DB_NAME)?;
    let ac_db = sled::open(Account::DB_NAME)?;

    let file = File::open(&filename).map_err(|_| "Error opening CSV file")?;
    // Use buffreader so the file is not loaded in memory all at once
    let filereader = BufReader::new(file);
    let mut csv_reader = ReaderBuilder::new()
        .trim(Trim::All)
        .has_headers(true)
        .from_reader(filereader);

    for result in csv_reader.deserialize::<Transaction>() {
        let mut tx: Transaction = result?;
        let mut acc = get_or_create_account(&ac_db, tx.client)?;

        match process_transaction(&tx_db, &mut acc, &mut tx) {
            Ok(()) => {
                insert_account(&ac_db, &acc)?;
            }
            Err(e) => eprintln!("Error processing transaction: {}", e),
        }
    }

    output_db_as_csv(&ac_db)?;
    cleanup();
    Ok(())
}

fn process_transaction(
    tx_db: &Db,
    acc: &mut Account,
    tx: &mut Transaction,
) -> Result<(), Box<dyn Error>> {
    match get_transaction(tx_db, &tx.tx)? {
        Some(mut updated_tx) => {
            if tx.tx_type == updated_tx.tx_type && tx.amount == updated_tx.amount {
                return Ok(()); // Idempotent transaction, nothing to do
            }

            // adding suffix to tx so they don't overwrite Deposits and Withdrawals,
            // which can be disputed later
            tx.tx.push_str(match tx.tx_type {
                TxType::Dispute => "-d",
                TxType::Resolve => "-r",
                TxType::Chargeback => "-c",
                _ => "",
            });

            match tx.tx_type {
                TxType::Deposit => tx.deposit(acc),
                TxType::Withdrawal => tx.withdrawal(acc),
                TxType::Dispute => updated_tx.dispute(acc),
                TxType::Resolve => updated_tx.resolve(acc),
                TxType::Chargeback => updated_tx.chargeback(acc),
            }

            insert_transaction(tx_db, &updated_tx)?;
        }
        None => {
            match tx.tx_type {
                TxType::Deposit => tx.deposit(acc),
                TxType::Withdrawal => tx.withdrawal(acc),
                TxType::Dispute | TxType::Resolve | TxType::Chargeback => return Ok(()),
            }
            insert_transaction(tx_db, &tx)?;
        }
    }
    Ok(())
}

fn get_or_create_account(db: &Db, client_id: u16) -> Result<Account, Box<dyn Error>> {
    // for each transaction, one account fetched or created
    // check if transaction with same tx (id) already stored
    match get_account(db, client_id)? {
        Some(acc) => Ok(acc),
        None => Ok(Account::new(client_id)),
    }
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
    db.insert(tx.tx.as_bytes(), serialized_data.as_bytes())?;
    db.flush()?;
    Ok(())
}

fn get_transaction(db: &Db, key: &String) -> Result<Option<Transaction>, Box<dyn Error>> {
    if let Some(serialized_data) = db.get(key.as_bytes())? {
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
        wtr.serialize((
            account.id,
            format!("{:.4}", account.available),
            format!("{:.4}", account.held),
            format!("{:.4}", account.total),
            account.locked,
        ))?;
    }

    wtr.flush()?;
    Ok(())
}

fn cleanup() {
    let _ = remove_dir_all(Account::DB_NAME);
    let _ = remove_dir_all(Transaction::DB_NAME);
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::ReaderBuilder;
    use sled::Config;
    use std::io::Cursor;

    #[test]
    fn test_process_transactions_in_memory() {
        // Sample CSV data in memory
        let csv_data = "\
            type,client,tx,amount\n\
            deposit,1,tx1,100.0\n\
            withdrawal,1,tx2,50.0\n";

        let tx_db = Config::new().temporary(true).open().unwrap();
        let ac_db = Config::new().temporary(true).open().unwrap();

        let mut csv_reader = ReaderBuilder::new()
            .trim(Trim::All)
            .has_headers(true)
            .from_reader(Cursor::new(csv_data));

        for result in csv_reader.deserialize::<Transaction>() {
            let mut tx: Transaction = result.unwrap();
            let mut acc = get_or_create_account(&ac_db, tx.client).unwrap();

            match process_transaction(&tx_db, &mut acc, &mut tx) {
                Ok(()) => {
                    insert_account(&ac_db, &acc).unwrap();
                }
                Err(e) => eprintln!("Error processing transaction: {}", e),
            }
        }

        // Check if account data is updated correctly
        let account = get_account(&ac_db, 1).unwrap().unwrap();
        assert_eq!(account.available, 50.0);
        assert_eq!(account.total, 50.0);
        assert_eq!(account.held, 0.0);
    }

    #[test]
    fn test_get_or_create_account_in_memory() {
        let db = Config::new().temporary(true).open().unwrap();

        // Creating a new account
        let account = get_or_create_account(&db, 1).unwrap();
        assert_eq!(account.id, 1);
        assert_eq!(account.total, 0.0);
        assert_eq!(account.available, 0.0);
        assert_eq!(account.held, 0.0);
        assert!(!account.locked);

        // Fetching an existing account
        insert_account(&db, &account).unwrap();
        let fetched_account = get_or_create_account(&db, 1).unwrap();
        assert_eq!(fetched_account.id, 1);
    }

    #[test]
    fn test_insert_and_get_account_in_memory() {
        let db = Config::new().temporary(true).open().unwrap();

        let account = Account {
            id: 1,
            total: 100.0,
            available: 100.0,
            held: 0.0,
            locked: false,
        };

        insert_account(&db, &account).unwrap();
        let fetched_account = get_account(&db, 1).unwrap().unwrap();

        assert_eq!(fetched_account.id, 1);
        assert_eq!(fetched_account.total, 100.0);
        assert_eq!(fetched_account.available, 100.0);
        assert_eq!(fetched_account.held, 0.0);
        assert!(!fetched_account.locked);
    }

    #[test]
    fn test_insert_and_get_transaction_in_memory() {
        let db = Config::new().temporary(true).open().unwrap();

        let transaction = Transaction {
            tx_type: TxType::Deposit,
            client: 1,
            tx: "tx1".to_string(),
            amount: 100.0,
            under_dispute: false,
        };

        insert_transaction(&db, &transaction).unwrap();
        let fetched_transaction = get_transaction(&db, &"tx1".to_string()).unwrap().unwrap();

        assert_eq!(fetched_transaction.tx_type, TxType::Deposit);
        assert_eq!(fetched_transaction.client, 1);
        assert_eq!(fetched_transaction.tx, "tx1");
        assert_eq!(fetched_transaction.amount, 100.0);
        assert!(!fetched_transaction.under_dispute);
    }

    #[test]
    fn test_output_db_as_csv_in_memory() {
        let db = Config::new().temporary(true).open().unwrap();

        let account = Account {
            id: 1,
            total: 100.0,
            available: 100.0,
            held: 0.0,
            locked: false,
        };

        insert_account(&db, &account).unwrap();

        // Redirect output to a buffer
        let mut buffer = Vec::new();
        {
            let mut writer = csv::Writer::from_writer(&mut buffer);
            writer
                .write_record(&["client", "available", "held", "total", "locked"])
                .unwrap();
            writer
                .serialize((
                    account.id,
                    format!("{:.4}", account.available),
                    format!("{:.4}", account.held),
                    format!("{:.4}", account.total),
                    account.locked,
                ))
                .unwrap();
            writer.flush().unwrap();
        }

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("client,available,held,total,locked"));
        assert!(output.contains("1,100.0000,0.0000,100.0000,false"));
    }
}

