use anyhow::Result;
use async_trait::async_trait;
use deadpool_postgres::tokio_postgres::{self, binary_copy::BinaryCopyInWriter};
use futures::{StreamExt, stream};
use tracing::{error, trace};

use super::traits::{PostgreSQL, SqlTypes};

/// An extension for asynchronous versions of INSERT & COPY for loading data to Postgres.
///
/// The data needs to implement a sql mapping, through that trait ['PostgreSQL'].
#[async_trait]
pub trait PgLoadExt {
    /// INSERT transaction.
    async fn insert_iter<'a, I, T>(&mut self, stmt: &'a str, collection: I) -> Result<()>
    where
        I: Iterator<Item = T> + Send + Sync,
        T: PostgreSQL + Send + Sync;

    /// COPY transactions cannot fail and still continue committing the rest of the data; any duplicate
    /// data (or any other failing circumstances) must be dealt with prior to the use of the `copy()` function.
    async fn copy<'a, I, T>(&mut self, stmt: &'a str, collection: I) -> Result<()>
    where
        I: Iterator<Item = T> + Send + Sync,
        T: SqlTypes + PostgreSQL + Send + Sync;
}

#[async_trait]
impl PgLoadExt for tokio_postgres::Client {
    async fn insert_iter<'a, I, T>(&mut self, stmt: &'a str, collection: I) -> Result<()>
    where
        I: Iterator<Item = T> + Send + Sync,
        T: PostgreSQL + Send + Sync,
    {
        // Start a transaction with a prepared statement.
        let stmt = self.prepare(stmt).await?;
        let tx = self.transaction().await?;

        // Stream the symbols & insert them to the database.
        let mut stream = stream::iter(collection.into_iter());
        while let Some(item) = stream.next().await {
            let stmt = &stmt;
            let tx = &tx;
            tx.execute(stmt, &item.sql_map()).await?;
        }
        trace!("{stmt:?} executed successfully");

        // Commit the transaction.
        tx.commit().await?;
        Ok(())
    }

    async fn copy<'a, I, T>(&mut self, stmt: &'a str, collection: I) -> Result<()>
    where
        I: Iterator<Item = T> + Send + Sync,
        T: SqlTypes + PostgreSQL + Send + Sync,
    {
        // Get a client from the Pool.
        let tx = self.transaction().await?;
        let sink = tx.copy_in(stmt).await?;
        let writer = BinaryCopyInWriter::new(sink, T::sql_types());
        futures::pin_mut!(writer); // writer must be pinned to use

        // Loop the collection & write to the `BinaryCopyInWriter`.
        // Possible async stream could go here, but copies are so quick this may be faster.
        for item in collection {
            match writer.as_mut().write(&item.sql_map()).await {
                Ok(_) => {}
                Err(e) => error!("Failed to copy {stmt:#?}: {e})"),
            }
        }
        trace!("{stmt:?} executed successfully");

        // Commit the transaction.
        writer.finish().await?;
        tx.commit().await?;
        Ok(())
    }
}

#[async_trait]
impl PgLoadExt for deadpool_postgres::Pool {
    async fn insert_iter<'a, I, T>(&mut self, stmt: &'a str, collection: I) -> Result<()>
    where
        I: Iterator<Item = T> + Send + Sync,
        T: PostgreSQL + Send + Sync,
    {
        let mut pg_client = self.get().await?;
        pg_client.insert_iter(stmt, collection).await?;
        Ok(())
    }

    async fn copy<'a, I, T>(&mut self, stmt: &'a str, collection: I) -> Result<()>
    where
        I: Iterator<Item = T> + Send + Sync,
        T: SqlTypes + PostgreSQL + Send + Sync,
    {
        let mut pg_client = self.get().await?;
        pg_client.copy(stmt, collection).await?;
        Ok(())
    }
}
