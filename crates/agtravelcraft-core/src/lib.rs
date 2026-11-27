#![forbid(unsafe_code)]

pub mod bus;
pub mod council;
pub mod proto;
pub mod skills;
pub mod store;
pub mod waystation;

use std::path::PathBuf;
use thiserror::Error;

pub const MAX_HOPS_CEILING: u8 = 4;
pub const MAX_COUNCIL_ROUNDS: u8 = 6;
pub const MAX_TURN_CHARS: usize = 2000;
pub const MAX_CAPSULE_BYTES: usize = 256 * 1024;

#[derive(Debug, Error)]
pub enum TravelError {
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("security policy violation: {0}")]
    SecurityViolation(String),

    #[error("access denied: {0}")]
    AccessDenied(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type TravelResult<T> = Result<T, TravelError>;

pub fn default_db_path() -> PathBuf {
    if let Ok(explicit) = std::env::var("AGTRAVELCRAFT_DB") {
        let t = explicit.trim();
        if !t.is_empty() {
            return PathBuf::from(t);
        }
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let mut p = PathBuf::from(local);
        p.push("AgTravelCraft");
        p.push("agtravelcraft.db");
        return p;
    }
    PathBuf::from("agtravelcraft.db")
}
