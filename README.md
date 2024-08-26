# Transaction Processing

This application, written in Rust, takes a CSV file containing transactions with fields Type, Client, Tx and Amount and executes the transactions on the respective Accounts (Clients). The output is a CSV-formatted string with the Accounts after all transactions are processed.

Transactions have 5 variant types: Deposit, Withdrawal, Dispute, Resolve and Chargeback. Each with its own behavior and rule. This software handles all of them.

## A few decisions

One of the first decisions made regarding this project was what crates to use. I have decided to go with 

* [clap](https://crates.io/crates/clap) for CLI parsing,
* [csv](https://crates.io/crates/csv) to process CSV input and output,
* [serde](https://crates.io/crates/serde) as base for Serialization/Deserialization,
* [serde_json](https://crates.io/crates/serde_json) to serialize to and from json as a intermediate format,
* [sled](https://crates.io/crates/sled) as a key-value DB in which to put records during execution.

Considering the resource management for this project, holding all the Transactions and Accounts in memory during execution is simply not feasible. So an embedded K/V database was used for both record types.
So during runtime, for each Transaction, an Account is either fetched or created, and then the transaction is processed. If the Transaction is not applicable (due to the type or other factor), it is skipped.
The input file is read via a Buffer Reader, in order to not load it entirely in memory at once. And this way it could also handle input from other sources (a TCP stream, for example).

Tests are not implemented (neither Unit Tests nor Integration Tests). But basic correctness is ensured by the type system. Errors are handled with pattern-matching in a few cases, and propagated in others. This approach was chosen in part to exemplify different ways to handle errors and variable cases.

Comments are reduced to places where it is important to document the reason behind a certain decision, instead of the details of implementation or functionality.

## Testing

Since unit tests are not implemented, I needed a csv file to use as input. I used Python's `random` module to create a CSV file with a number of with fake transactions. You can find it in `data.csv`

## Running the project

Having [Rust installed](https://www.rust-lang.org/tools/install), just run:

```shell
cargo run -- data.csv
```

the output is set to `stdout` by default. To change it to a file, you can redirect it in the CLI:

```shell
cargo run -- data.csv > output.csv
``` 

If any questions come up, feel free to reach out to me.
