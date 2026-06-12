# Pipelines for ETL
Rust framework for speeding up ETL processes. Currently supports:
- PostgreSQL (copying, inserting, selecting iterables)
- HTTP (just a `reqwest` wrapper with a `download_file()` addition)
- filestore helpers (reading json, csv, tsv or `stringify_jpg()`)

# Getting started
The package revolves mostly around the `petl::Connections` struct:

```{rust}
let conn = petl::Connections::new(
    // Returns
    // EXAMPLE_USERNAME,
    // EXAMPLE_PASSWORD,
    // EXAMPLE_ADDRESS,
    // EXAMPLE_PORT &
    // EXAMPLE_DBNAME from env variables, then build a `deadpool_postgres::Pool`.
    //
    // This section of the struct is under `conn.pg`.
    petl::PgPoolConfig::from_env("EXAMPLE")?.create_pool()?,

    // `conn.http` is built via the `reqwest` crate, but this crate is so good there's almost
    // nothing worth changing - it's mostly convenience.
    reqwest::ClientBuilder::new()
        .user_agent("example@example.com")
        .default_headers(headers)
        .build()?,
);
```

# Usage
Once connections established, they can be accessed to speed up pipelines.
```{rust}
use petl::prelude::*; // <-- contains the generic traits required across HTTP & PG processes.

// Let's say we want to deserialize the following struct from a json endpoint,
// from the URL: <https://api.binance.com/api/v1/ticker/allBookTickers>.
#[derive(serde::Deserialize)]
struct Ticker {
    symbol: String,
}

// And we're gonna transform it ready for a postgres table as the output.
// `petl::PostgreSQL` automatically transforms rust datatypes to postgres datatypes.
#[derive(Deserialize, petl::PostgreSQL)]
pub struct DimAsset {
    pub symbol: String,
    pub name: Option<String>,
    pub industry: String,
    pub nation: String,
    pub class: String,
    pub meta: Option<String>,
}

// HTTP just does the same as reqwest does usually
let tickers: Vec<DimAsset> = conn
    .http
    .get("https://api.binance.com/api/v1/ticker/allBookTickers")
    .send()
    .await?
    .json::<Vec<Ticker>>()
    .await?
    .into_iter()
    .map(|t| DimAsset {
        symbol: t.symbol,
        name: None,
        industry: "CRYPTOCURRENCY".to_string(),
        nation: "WEB3".to_string(),
        class: "crypto".to_string(),
        meta: None,
    })
    .collect();
    
// But the postgres element, pg, **really** speeds things up
conn.pg
    .insert_iter( // <-- inserts for an iterable collection of the same element
        include_str!("../../../scripts/insert_into_dim_assets.sql"), // <-- pull a sql file at compile time
        tickers.iter(),
        // ^^^ since our datatype has the derive macro `petl::PostgreSQL`, it
        //     doesn't require any additional info.
    )
    .await?;
```
