pub mod compress;
pub mod extract;
pub mod load;
pub mod traits;

pub trait Postgres: extract::PgExtractExt + load::PgLoadExt {}
impl<T> Postgres for T where T: extract::PgExtractExt + load::PgLoadExt {}
