use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    username: String,
    password: String,
    port: u16,
    address: String,
    dbname: String,
}

impl Config {
    /// Create a config from raw inputs.
    ///
    /// *NOT RECOMMENDED*; `.env` files, `keyrings` or `vaults` are much more security-effective.
    pub fn new(username: &str, password: &str, port: u16, address: &str, dbname: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
            port,
            address: address.to_string(),
            dbname: dbname.to_string(),
        }
    }

    /// Read login details (username, password, port, address, dbname) from Environment Variables.
    ///
    /// ### e.g.
    ///
    /// In the `.env` file:
    /// ```
    /// MYEXAMPLE_USERNAME="user"
    /// MYEXAMPLE_PASSWORD="examplepassword"
    /// MYEXAMPLE_PORT=5432
    /// MYEXAMPLE_ADDRESS="localhost"
    /// MYEXAMPLE_DBNAME="postgres"
    /// ```
    ///
    /// Then, through rust:
    /// ```{rust}
    /// let pg_pool = petl::postgres::config::Config::from_env("MYEXAMPLE") // provide the prefix
    ///     .create_pool()
    ///     .unwrap();
    /// ```
    pub fn from_env(env_var_prefix: &str) -> Result<Self> {
        let login: Self = config::Config::builder()
            .add_source(config::Environment::with_prefix(env_var_prefix).separator("_"))
            .build()?
            .try_deserialize()?;
        Ok(login)
    }

    /// Create a `deadpool_postgres::Pool` the `Config`.
    ///
    /// NOTE: this currently assumes **no TLS**.
    pub fn create_pool(self) -> Result<deadpool_postgres::Pool> {
        let mut pg_config = deadpool_postgres::Config::new();
        pg_config.user = Some(self.username);
        pg_config.password = Some(self.password);
        pg_config.port = Some(self.port);
        pg_config.host = Some(self.address);
        pg_config.dbname = Some(self.dbname);

        use deadpool_postgres::Runtime;
        let pool = pg_config.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)?;
        Ok(pool)
    }
}
