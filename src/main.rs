use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::Duration;

use rusqlite::{Connection, Result};
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
}

#[derive(Debug, Serialize, Deserialize)]
struct Record {
    id: String,
    tx: Vec<u8>,
}

fn main() -> Result<(), MyError> {
    let connection = Connection::open("add/mempool.sqlite")?;
    let mut last_id = String::new();

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .append(true)
        .open("add/output.json")
        .unwrap();

    loop {
        let mut stmt = connection.prepare("SELECT * FROM mempool WHERE tx > ?")?;
        let rows = stmt.query_map([&last_id], |row| {
            Ok(Record {
                id: row.get(0)?,
                tx: row.get(11)?,
            })
        })?;

        for row in rows {
            let record = row?;
            let serialized = serde_json::to_string(&record.tx)?;

            file.write_all(serialized.as_bytes())?;
            file.write_all(b"\n")?;

            last_id = record.id;
            println!("{:?}", record.tx)
        }

        thread::sleep(Duration::from_secs(1));
    }
}
