use anyhow::Result;
use petl::PgPoolConfig;
use petl::prelude::*;

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
    // create a new `Session`
    // it uses a simple `reqwest::Client`
    let mut ssn = Session::new(
        PgPoolConfig::new("postgres", "postgres", 5432, "localhost", "postgres").create_pool()?,
        reqwest::ClientBuilder::new()
            .user_agent("example@example.com")
            .build()?,
    );

    // ping pg db
    let _ping: i32 = ssn.pg.select("SELECT 1", &[]).await?;
    println!("pinged!");

    // set up a table
    ssn.pg
        .get()
        .await?
        .query("DROP TABLE IF EXISTS prices", &[])
        .await?;
    ssn.pg
        .get()
        .await?
        .query(
            r#"
            CREATE TABLE IF NOT EXISTS prices (
                timestamps BIGINT
            )
            "#,
            &[],
        )
        .await?;
    println!("prices table created");

    // nvidia prices from yahoo finance
    let url: &str = "https://query2.finance.yahoo.com/v8/finance/chart/NVDA?range=50y&interval=1d";
    let timestamps: YahooFinance = ssn.http.get(url).send().await?.json().await?;
    ssn.pg
        .insert_iter(
            "INSERT INTO prices (timestamps) VALUES ($1)",
            timestamps.chart.result.meta.timestamp.iter(),
        )
        .await?;

    let returned: Vec<i64> = ssn
        .pg
        .select_collection("SELECT timestamps FROM prices", &[], |row| row.get(0))
        .await?;
    println!("{} rows", returned.len());

    Ok(())
}
