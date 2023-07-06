use std::{
    fs::File,
    io::{Read, Write},
    str, thread,
    time::Duration,
};

use blockstack_lib::{
    chainstate::stacks::StacksTransaction,
    core::mempool::MemPoolTxInfo,
    util_lib::db::{self, FromRow},
};

use rusqlite::{Connection, Result};
use thiserror::Error;

const OUTPUT_JSON: &str = "add/output.json";
const LAST_ACCEPT_TIME: &str = "add/last_accept_time";
const BATA_BASE: &str = "add/mempool.sqlite";

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
}

fn main() -> Result<(), MyError> {
    env_logger::init();

    let connection = Connection::open(BATA_BASE)?;

    let mut last_accept_time: u64 = {
        File::open(LAST_ACCEPT_TIME)
            .map_err(Into::<MyError>::into)
            .and_then(|mut file| {
                let mut buf = String::new();
                file.read_to_string(&mut buf)
                    .map(|_| buf)
                    .map_err(Into::into)
            })
            .and_then(|str| str.parse().map_err(Into::into))
            .unwrap_or_else(|err| {
                log::error!("Failed to read last accept time: {err}\nAssuming last accept time as 0");
                0
            })
    };

    log::debug!("last_accept_time = {last_accept_time}");

    let mut transactions: Vec<StacksTransaction> = {
        File::open(OUTPUT_JSON)
            .map_err(Into::<MyError>::into)
            .and_then(|file| serde_json::from_reader(file).map_err(Into::into))
            .unwrap_or_else(|err| {
                log::error!("failed to load transactions: {err}\nFile will be recreated\nAssuming last accept time as 0");
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
                .open(LAST_ACCEPT_TIME)?;
            
            write!(f, "{last_accept_time}")?;

            let f = File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(OUTPUT_JSON)?;

            serde_json::to_writer_pretty(f, &transactions)?;

            dirty = false;
        }

        thread::sleep(Duration::from_secs(5));
    }
}
