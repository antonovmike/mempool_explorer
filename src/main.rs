use std::{
    fs::File,
    io::{Read, Write},
    thread,
    time::Duration,
};

use blockstack_lib::{
    chainstate::stacks::StacksTransaction,
    core::mempool::MemPoolTxInfo,
    util_lib::db::FromRow,
};
use clap::Parser;
use rusqlite::{Connection, Result};

use crate::error_handler::MyError;
use crate::file_processing::*;

mod error_handler;
mod file_processing;

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

        let mut part_of_file_name = String::new();

        while let Some(row) = rows.next()? {
            let tx_info = MemPoolTxInfo::from_row(row)?;

            let filename = format!("add/{}.json", part_of_file_name);

            let smart_contract = SmartContract {
                name_of_smart_contract: tx_info.clone(),
                tx_info_tx: tx_info.clone().tx,
                filename: filename.clone(),
            };

            part_of_file_name = SmartContract::contract_name(&smart_contract);

            if tx_info.metadata.accept_time > last_accept_time {
                last_accept_time = tx_info.metadata.accept_time;
            }

            transactions.push(tx_info.tx);

            separate_files(&transactions, smart_contract);

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
