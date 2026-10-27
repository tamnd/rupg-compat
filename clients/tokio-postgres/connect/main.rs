//! The connect program of the connect suite, with tokio-postgres.
//!
//! tokio-postgres does not read the libpq environment, so the program reads it. OpenSSL checks the certificate of the server with the root certificate in PGSSLROOTCERT and checks the host name. The program writes the row of the query on one line, with one space between the values.

use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use tokio_postgres::Config;
use tokio_postgres::config::{ChannelBinding, SslMode};

fn env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("{key} is not set"))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tls = SslConnector::builder(SslMethod::tls())?;
    tls.set_ca_file(env("PGSSLROOTCERT"))?;
    let mut config = Config::new();
    config
        .host(env("PGHOST"))
        .port(env("PGPORT").parse()?)
        .user(env("PGUSER"))
        .password(env("PGPASSWORD"))
        .dbname(env("PGDATABASE"))
        .ssl_mode(SslMode::Require)
        .channel_binding(ChannelBinding::Require);
    let (client, connection) = config.connect(MakeTlsConnector::new(tls.build())).await?;
    let task = tokio::spawn(connection);
    let row = client
        .query_one(
            "SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()",
            &[],
        )
        .await?;
    let (user, ssl, version): (String, bool, String) = (row.get(0), row.get(1), row.get(2));
    println!("{user} {} {version}", if ssl { "t" } else { "f" });
    drop(client);
    task.await??;
    Ok(())
}
