use std::{
    fs::File,
    io::{Read, Write},
    thread,
    time::Duration, num::ParseIntError,
};

use blockstack_lib::{
    chainstate::stacks::StacksTransaction,
    core::mempool::MemPoolTxInfo,
    util_lib::db::{self, FromRow},
};
use clap::Parser;
use rusqlite::{Connection, Result};
use thiserror::Error;

const DEFAULT_OUTPUT_FILE_NAME: &str = "output.json";
const DEFAULT_LAST_ACCEPT_TIME_FILE_NAME: &str = "last_accept_time";

#[derive(Parser, Debug)]
struct Args {
    /// Path to mempool DB
    mempool_db: String,

    /// Path to last_accept_time file
    #[clap(short = 't', default_value_t = DEFAULT_LAST_ACCEPT_TIME_FILE_NAME.into())]
    last_accept_file: String,

    /// Path to the output JSON file
    #[clap(short = 'o', default_value_t = DEFAULT_OUTPUT_FILE_NAME.into())]
    output: String,
}

struct Payload {
    contract_call: ContractCall,
}

struct ContractCall {
    address: Address,
    contract_name: String,
    function_name: String,
    function_args: String,
}

struct Address {
    version: i32,
    bytes: String,
}

#[derive(Error, Debug)]
enum MyError {
    #[error("database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("file error: {0}")]
    RusQLite(#[from] std::io::Error),
    #[error("stack core error: {0}")]
    StacksCoreErr(#[from] db::Error),
    #[error("parse error: {0}")]
    ParseError(#[from] std::num::ParseIntError),
    #[error("path expansion error: {0}")]
    ExpandError(String),
}

fn main() -> Result<(), MyError> {
    let args = Args::parse();

    let last_accept_time_file_path = &*shellexpand::full(&args.last_accept_file)
        .map_err(|e| MyError::ExpandError(format!("{e}")))?;
    let db_path =
        &*shellexpand::full(&args.mempool_db).map_err(|e| MyError::ExpandError(format!("{e}")))?;
    let output_file_path =
        &*shellexpand::full(&args.output).map_err(|e| MyError::ExpandError(format!("{e}")))?;

    env_logger::init();

    let connection = Connection::open(db_path)?;

    let mut last_accept_time: u64 = {
        File::open(last_accept_time_file_path)
            .map_err(Into::<MyError>::into)
            .and_then(|mut file| {
                let mut buf = String::new();
                file.read_to_string(&mut buf)
                    .map(|_| buf)
                    .map_err(Into::into)
            })
            .and_then(|str| str.parse().map_err(Into::into))
            .unwrap_or_else(|err| {
                log::warn!(
                    "Failed to read last accept time: {err}\nAssuming last accept time as 0"
                );
                0
            })
    };

    log::debug!("last_accept_time = {last_accept_time}");

    let mut transactions: Vec<StacksTransaction> = {
        File::open(output_file_path)
            .map_err(Into::<MyError>::into)
            .and_then(|file| serde_json::from_reader(file).map_err(Into::into))
            .unwrap_or_else(|err| {
                log::warn!("failed to load transactions: {err}\nFile will be recreated\nAssuming last accept time as 0");
                last_accept_time = 0;
                vec![]
            })
    };

    let mut dirty = false;

    loop {
        let mut stmt = connection.prepare("SELECT * FROM mempool WHERE accept_time > ?")?;
        let mut rows = stmt.query([last_accept_time as i64])?;

        while let Some(row) = rows.next()? {
            let tx_info = MemPoolTxInfo::from_row(row)?;

            let name_of_smart_contract = tx_info.clone();
            contract_name(name_of_smart_contract);

            if tx_info.metadata.accept_time > last_accept_time {
                last_accept_time = tx_info.metadata.accept_time;
            }

            transactions.push(tx_info.tx);
            dirty = true;
        }

        if dirty {
            let mut f = File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(last_accept_time_file_path)?;

            log::debug!("last_accept_time is changed to {last_accept_time}");
            write!(f, "{last_accept_time}")?;

            let f = File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(output_file_path)?;

            serde_json::to_writer_pretty(f, &transactions)?;

            dirty = false;
        }

        thread::sleep(Duration::from_secs(5));
    }
}

fn contract_name(name_of_smart_contract: MemPoolTxInfo) {
    let string = format!("{:?}", name_of_smart_contract.tx.payload);
    let substring = "ContractName(\"";
    let split = string.splitn(2, substring);
    let string_2 = match split.last() {
        Some(trimmed) => trimmed,
        None => &string,
    };

    let substring = "\"), function_name";
    let mut split = string_2.splitn(2, substring);
    let string_3 = match split.next() {
        Some(trimmed) => trimmed,
        None => &string,
    };
    println!("\tTEST\n{string_3}");
}