/// Pull together several connection frameworks (like `postgres` and `http`).
pub struct Connections {
    pub pg: deadpool_postgres::Pool,
    pub http: reqwest::Client,
    // pub cache: rusqlite::Connection,
}

impl Connections {
    /// Create a new `Session`, indicating which API endpoints you want to call upon.
    ///
    /// e.g. a process containing the `postgres` & `http` features.
    pub fn new(pg: deadpool_postgres::Pool, http: reqwest::Client) -> Self {
        Self { pg: pg, http }
    }
}

/// Default settings for a localhost postgres database and default http client.
impl Default for Connections {
    fn default() -> Self {
        Self {
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
                .expect("Failed to create default pg pool")
            },
            http: reqwest::ClientBuilder::new()
                .user_agent("example@example.com")
                .build()
                .expect("Failed to build default HTTP client"),
        }
    }
}
