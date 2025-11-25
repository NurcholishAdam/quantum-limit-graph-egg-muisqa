// crates/limit-storage/src/lib.rs
pub mod storage;
pub mod db_storage;

pub use storage::{Storage, FileStorage};
pub use db_storage::KVStorage;

#[cfg(feature = "sqlite")]
pub use db_storage::SqliteStorage;

#[cfg(feature = "postgres")]
pub use db_storage::PostgresStorage;
