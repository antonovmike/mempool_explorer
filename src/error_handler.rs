use blockstack_lib::util_lib::db;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
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