use anyhow::Result;
use async_trait::async_trait;
// use deadpool_postgres::tokio_postgres::Row;
// use deadpool_postgres::tokio_postgres::types::{FromSql, ToSql};
use tokio_postgres::Row;
use tokio_postgres::types::{FromSql, ToSql};
use tracing::{debug, error, trace};

/// An extension for shortcutting some Postgres-scraping protocols.
#[async_trait]
pub trait PgExtractExt {
    /// If a single entity exists, return it.
    async fn select<'a, T>(&self, fetch_stmt: &'a str, params: &[&(dyn ToSql + Sync)]) -> Result<T>
    where
        T: for<'b> FromSql<'b>;

    /// If a single entity exists, return it; if not, insert it.
    async fn select_else_insert<'a, T>(
        &self,
        fetch_stmt: &'a str,
        insert_stmt: &'a str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<T>
    where
        T: for<'b> FromSql<'b> + 'static;

    /// Return some collection from the database.
    async fn select_collection<C, T, F, 'a>(
        &self,
        fetch_stmt: &'a str,
        params: &[&(dyn ToSql + Sync)],
        f: F,
    ) -> Result<C>
    where
        C: FromIterator<T>,
        T: Send,
        F: FnMut(&Row) -> T + Send;
}

#[async_trait]
impl PgExtractExt for tokio_postgres::Client {
    async fn select<'a, T>(&self, fetch_stmt: &'a str, params: &[&(dyn ToSql + Sync)]) -> Result<T>
    where
        T: for<'b> FromSql<'b>,
    {
        trace!(fetch_stmt = %fetch_stmt, "Fetching data for query");
        let data: Row = match self.query_one(fetch_stmt, params).await {
            Ok(response) => response,
            Err(e) => {
                error!(stmt = %fetch_stmt, "{e}");
                return Err(anyhow::anyhow!(e));
            }
        };

        Ok(data.get(0))
    }

    async fn select_else_insert<'a, T>(
        &self,
        fetch_stmt: &'a str,
        insert_stmt: &'a str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<T>
    where
        T: for<'b> FromSql<'b>,
    {
        // Attempt to find the Source PK in the existing table.
        let data: Row = match self.query_one(fetch_stmt, params).await {
            Ok(response) => response,

            // If no PK is found, insert a new one, and reattempt to find it.
            Err(e) => {
                debug!("Did not find data for query: {fetch_stmt} - inserting data instead: {e}");
                self.query_one(insert_stmt, params).await?;

                match self.query_one(fetch_stmt, params).await {
                    Ok(second_response) => second_response,
                    Err(e) => {
                        error!(fetch_stmt = %fetch_stmt, insert_stmt = %insert_stmt, "Failed to insert and retrieve new data");
                        return Err(anyhow::anyhow!(e));
                    }
                }
            }
        };
        Ok(data.get(0))
    }

    async fn select_collection<'a, C, T, F>(
        &self,
        fetch_stmt: &'a str,
        params: &[&(dyn ToSql + Sync)],
        f: F,
    ) -> Result<C>
    where
        C: FromIterator<T>,
        T: Send,
        F: FnMut(&Row) -> T + Send,
    {
        // Return the collection from the pg database.
        let data: Vec<Row> = match self.query(fetch_stmt, params).await {
            Ok(response) => response,
            Err(e) => {
                error!(fetch_stmt = %fetch_stmt, "Failed to fetch collection");
                return Err(anyhow::anyhow!(e));
            }
        };

        // Transform the array of [`tokio_postgres::Row`] with some closure.
        let output: C = data.iter().map(f).collect();
        Ok(output)
    }
}

#[async_trait]
impl PgExtractExt for deadpool_postgres::Pool {
    async fn select<'a, T>(&self, fetch_stmt: &'a str, params: &[&(dyn ToSql + Sync)]) -> Result<T>
    where
        T: for<'b> FromSql<'b>,
    {
        let pg_client = self.get().await?;
        Ok(pg_client.select(fetch_stmt, params).await?)
    }

    async fn select_else_insert<'a, T>(
        &self,
        fetch_stmt: &'a str,
        insert_stmt: &'a str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<T>
    where
        T: for<'b> FromSql<'b> + 'static,
    {
        let pg_client = self.get().await?;
        let data = pg_client
            .select_else_insert(fetch_stmt, insert_stmt, params)
            .await?;
        Ok(data)
    }

    async fn select_collection<'a, C, T, F>(
        &self,
        fetch_stmt: &'a str,
        params: &[&(dyn ToSql + Sync)],
        f: F,
    ) -> Result<C>
    where
        C: FromIterator<T>,
        T: Send,
        F: FnMut(&Row) -> T + Send,
    {
        let pg_client = self.get().await?;
        let data = pg_client.select_collection(fetch_stmt, params, f).await?;
        Ok(data)
    }
}
