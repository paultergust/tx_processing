# Transaction Processing

This Rust application processes a CSV file containing financial transactions, applies them to corresponding accounts and outputs the updated account states as a CSV-formatted string.

## Table of Contents

- [Overview](#overview)
- [Crate Selection](#crate-selection)
- [Design Decisions](#design-decisions)
- [Running the project](#running-the-project)

## Overview

This application executes various types of financial transactions (Deposit, Withdrawal, Dispute, Resolve, and Chargeback) on client accounts, adhering to specific rules for each transaction type. The result is a summary of all accounts after processing.

## Crate Selection

The following crates were selected to build this project:

- **[clap](https://crates.io/crates/clap)**: Used for command-line interface (CLI) parsing to easily handle user inputs.
- **[csv](https://crates.io/crates/csv)**: Provides utilities for reading and writing CSV files, which is the format for both input and output data.
- **[serde](https://crates.io/crates/serde)**: A framework for serializing and deserializing Rust data structures.
- **[serde_json](https://crates.io/crates/serde_json)**: Facilitates the serialization to and from JSON as an intermediate format for storage in the key-value database.
- **[sled](https://crates.io/crates/sled)**: A high-performance embedded key-value store used to manage the accounts and transactions on disk.

## Design Decisions

### Resource Management

To avoid holding all transactions and accounts in memory, which is not feasible for large datasets, the application uses **sled** as an embedded key-value database to store accounts and transactions. Each transaction fetches or creates the associated account, processes the transaction, and updates the account state in the database.

### Input Handling

The input CSV file is processed using a `BufReader` to prevent loading the entire file into memory, which also allows the application to handle streaming data from various sources, such as TCP streams.

### Error Handling

Error handling is implemented through a combination of pattern matching and error propagation. The application demonstrates different strategies for handling errors in Rust:
- Pattern matching is used where immediate action is required.
- Error propagation (`?` operator) is used to defer error handling to the caller.

### Unit Tests

There are a few unit tests implemented. You can run them with:

```shell
cargo test
```

### Manual Tests

I used Python's `random` module to create a CSV file with a number of fake transactions. You can find it in `data.csv`

## Running the project

Having [Rust installed](https://www.rust-lang.org/tools/install), just run:

```shell
cargo run -- data.csv
```

The output is set to `stdout` by default. To change it to a file, you can redirect it in the CLI:

```shell
cargo run -- data.csv > output.csv
```

If any questions come up, feel free to reach out to me.
