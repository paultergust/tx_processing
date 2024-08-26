mod transaction;
mod account;

use std::{fs::File, io::BufReader};

use clap::Parser;
use csv::{ReaderBuilder, Trim};

use crate::transaction::Transaction;

#[derive(Parser)]
struct Cli {
    filepath: String,
}

fn main() {
    let cli = Cli::parse();
    let filepath = cli.filepath;
    process_transactions(filepath);
}

fn process_transactions(filename: String){
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
        println!("{:?}", result);
    }
}
