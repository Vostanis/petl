//! # Piped ETL
//!
//! Framework for inter-connection data processes, focused around the [`Connections`] struct,
//! and includes useful [`fs`] functions.

pub mod prelude {
    pub use crate::PostgreSQL;
    pub use crate::connections::Connections;
    pub use crate::http::extract::HttpExtractExt;
    pub use crate::postgres::{extract::PgExtractExt, load::PgLoadExt};
}

// Re-exports.
pub use connections::Connections;
pub use postgres::config::Config as PgPoolConfig;

// Re-export for `PostgreSQL` macro.
pub use petl_macros::PostgreSQL;

/// Re-export of `tokio_postgres::types::ToSql` (originally from `postgres_types`).
pub use tokio_postgres::types::ToSql;

/// Focal point of the crate - container for all connections.
pub mod connections;

/// Useful filestore-management functions.
pub mod fs;

pub mod http;
pub mod postgres;
