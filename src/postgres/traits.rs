use deadpool_postgres::tokio_postgres::types::{ToSql, Type};

/// Provide a SQL mapping for the item struct.
///
/// See [`tokio_postgres::types::ToSql`] for more detail.
pub trait PostgreSQL {
    fn sql_map(&self) -> Vec<&(dyn ToSql + Sync)>;
}

/// Auto-implementation for all values that impl ToSql + Sync.
impl<T: ToSql + Sync> PostgreSQL for &T {
    fn sql_map(&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![self]
    }
}

/// Provide the SQL types required.
///
/// See [`tokio_postgres::types::ToSql`] for more detail.
pub trait SqlTypes {
    fn sql_types() -> &'static [Type];
}
