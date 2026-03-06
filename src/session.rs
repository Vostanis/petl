// use anyhow::Result;

// PostgreSQL ----------------------
use super::postgres::Postgres;
#[cfg(not(feature = "postgres"))]
struct NoPg;
#[cfg(not(feature = "postgres"))]
impl Postgres for NoPg {}

// HTTP ----------------------------
use super::http::Http;
#[cfg(not(feature = "http"))]
struct NoHttp;
#[cfg(not(feature = "http"))]
impl Http for NoHttp {}

/// Pull together several connection frameworks (like `postgres` and `http`).
pub struct Session<P: Postgres, H: Http> {
    // PhantomData used to pass compiler checks on generics (i.e. P: Postgres)
    // see struct definition above
    #[cfg(feature = "postgres")]
    pub pg: P,
    #[cfg(not(feature = "postgres"))] // PhantomData when feature not included
    _no_pg: std::marker::PhantomData<P>,

    #[cfg(feature = "http")]
    pub http: H,
    #[cfg(not(feature = "http"))]
    _no_http: std::marker::PhantomData<H>,
    // #[cfg(feature = "sqlite")]
    // pub cache: Cache,
    // #[cfg(not(feature = "sqlite"))]
    // _no_cache: std::marker::PhantomData<Cache>,
}

impl<P: Postgres, H: Http> Session<P, H> {
    /// Create a new `Session`, indicating which API endpoints you want to call upon.
    ///
    /// e.g. a process containing the `postgres` & `http` features.
    pub fn new(#[cfg(feature = "postgres")] pg: P, #[cfg(feature = "http")] http: H) -> Self {
        Self {
            #[cfg(feature = "postgres")]
            pg: pg,
            #[cfg(not(feature = "postgres"))]
            _no_pg: NoPg {},
            #[cfg(feature = "http")]
            http,
            #[cfg(not(feature = "http"))]
            _no_http: NoHttp {},
        }
    }

    /// Alter the postgres connection.
    #[cfg(feature = "postgres")]
    pub fn mut_pg(&mut self, pg: P) {
        self.pg = pg;
    }

    /// Alter the http connection.
    #[cfg(feature = "http")]
    pub fn mut_http(&mut self, http: H) {
        self.http = http;
    }
}

/// Default settings for a localhost postgres database and default http client.
#[cfg(all(feature = "postgres", feature = "http"))]
impl Default for Session<deadpool_postgres::Pool, reqwest::Client> {
    fn default() -> Session<deadpool_postgres::Pool, reqwest::Client> {
        Session {
            #[cfg(feature = "postgres")]
            pg: {
                let mut conf = deadpool_postgres::Config::new();
                conf.user = Some("postgres".to_string());
                conf.password = Some("postgres".to_string());
                conf.dbname = Some("postgres".to_string());
                conf.host = Some("localhost".to_string());
                conf.port = Some(5432);
                conf.create_pool(
                    Some(deadpool_postgres::Runtime::Tokio1),
                    tokio_postgres::NoTls,
                )
                .expect("failed to create default postgres pool")
            },
            #[cfg(not(feature = "postgres"))]
            _no_pg: NoPg {},

            #[cfg(feature = "http")]
            http: reqwest::Client::new(),
            #[cfg(not(feature = "http"))]
            _no_pg: NoHttp {},
        }
    }
}
