use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::Duration;

use blockstack_lib::{core::mempool::MemPoolTxInfo, util_lib::db::{FromRow, self}};

use rusqlite::{Connection, Result, NO_PARAMS};
use serde_derive::*;
use thiserror::Error;

#[derive(Error, Debug)]
enum MyError {
    #[error("database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("file error: {0}")]
    RusQLite(#[from] std::io::Error),
    #[error("file error: {0}")]
    StacksCoreErr(#[from] db::Error)
}

#[derive(Debug, Serialize, Deserialize)]
struct Record {
    accept_time: String,
    tx: Vec<u8>,
}

fn main() -> Result<(), MyError> {
    let connection = Connection::open("add/mempool.sqlite")?;
    // let mut stmt = connection.prepare("SELECT * FROM mempool")?;
    // let mut rows = stmt.query(NO_PARAMS)?;
    // let mut transactions = vec![];

    // while let Some(row) = rows.next()? {
    //     transactions.push(MemPoolTxInfo::from_row(row)?);
    // }

    // dbg!(transactions);

    let mut last_accept_time: u64 = 1687841601;

    loop {
        let mut stmt = connection.prepare("SELECT * FROM mempool WHERE accept_time > ?")?;
        let mut rows = stmt.query([last_accept_time as i64])?;

        while let Some(row) = rows.next()? {
            let tx_info = MemPoolTxInfo::from_row(row)?;

            if tx_info.metadata.accept_time > last_accept_time {
                last_accept_time = tx_info.metadata.accept_time;
            }

            dbg!(tx_info.tx);
        }

        thread::sleep(Duration::from_secs(5));
    }

    Ok(())

    // let mut last_id = String::new();

    // let mut file = OpenOptions::new()
    //     .write(true)
    //     .create(true)
    //     .truncate(false)
    //     .append(true)
    //     .open("add/output.json")
    //     .unwrap();

    // loop {
    //     let mut stmt = connection.prepare("SELECT * FROM mempool WHERE tx > ?")?;
    //     let rows = stmt.query_map([&last_id], |row| {
    //         Ok(Record {
    //             id: row.get(0)?,
    //             tx: row.get(11)?,
    //         })
    //     })?;

    //     for row in rows {
    //         let record = row?;
    //         let serialized = serde_json::to_string(&record.tx)?;

    //         file.write_all(serialized.as_bytes())?;
    //         file.write_all(b"\n")?;

    //         last_id = record.id;
    //         println!("{:?}", record.tx)
    //     }

    //     thread::sleep(Duration::from_secs(5));
    // }
}
