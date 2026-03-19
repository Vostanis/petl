pub mod prelude {
    pub use crate::PostgreSQL;
    pub use crate::http::extract::HttpExtractExt;
    pub use crate::postgres::{extract::PgExtractExt, load::PgLoadExt};
    pub use crate::session::Session;
}

pub use session::Session;

pub use postgres::config::Config as PgPoolConfig;

pub mod postgres;

// Re-export for `PostgreSQL` macro.
pub use macros::PostgreSQL;
pub use tokio_postgres::types::ToSql;

pub mod http;

pub mod fs;

/// Focal point of the crate - container for all connections.
pub mod session;

/// Focal point of the crate - container for all connections.
pub mod connections;
pub use connections::Connections;
