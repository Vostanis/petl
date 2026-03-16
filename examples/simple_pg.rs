use anyhow::Result;
use petl::postgres::{config::Config, extract::PgExtractExt, load::PgLoadExt};
use petl::session::Session;

// + SCHEMA -------------------------

#[derive(Debug, serde::Deserialize)]
struct YahooFinance {
    chart: Chart,
}

#[derive(Debug, serde::Deserialize)]
struct Chart {
    result: YahooResult,
}

#[derive(Debug, serde::Deserialize)]
struct YahooResult {
    meta: Meta,
}

#[derive(Debug, serde::Deserialize)]
struct Meta {
    timestamp: Vec<i64>,
}

// +---------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    // Using deadpool-postgres::Pool
    let pg_pool =
        Config::new("postgres", "postgres", 5432, "localhost", "postgres").create_pool()?;

    // create a new `Session`
    // it uses a simple `reqwest::Client`
    let mut sx = Session::new(
        pg_pool,
        reqwest::ClientBuilder::new()
            .user_agent("example@example.com")
            .build()?,
    );

    // ping pg db
    let _ping: i32 = sx.pg.select("SELECT 1", &[]).await?;
    println!("pinged!");

    // set up a table
    sx.pg
        .get()
        .await?
        .query(
            "
            DROP TABLE IF EXISTS prices
        ",
            &[],
        )
        .await?;
    sx.pg
        .get()
        .await?
        .query(
            "
            CREATE TABLE IF NOT EXISTS prices (
                timestamps BIGINT
            )
        ",
            &[],
        )
        .await?;
    println!("prices table created");

    // nvidia prices from yahoo finance
    let url: &str = "https://query2.finance.yahoo.com/v8/finance/chart/NVDA?range=50y&interval=1d";
    let timestamps: YahooFinance = sx.http.get(url).send().await?.json().await?;
    sx.pg
        .insert_iter(
            "INSERT INTO prices (timestamps) VALUES ($1)",
            timestamps.chart.result.meta.timestamp.iter(),
        )
        .await?;

    let returned: Vec<i64> = sx
        .pg
        .select_collection("SELECT timestamps FROM prices", &[], |row| row.get(0))
        .await?;
    println!("{} rows", returned.len());

    Ok(())
}
