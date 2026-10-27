//! The connect program of the connect suite, with sqlx.
//!
//! `PgConnectOptions::new` reads PGHOST, PGPORT, PGUSER, PGPASSWORD, PGDATABASE, PGSSLMODE and PGSSLROOTCERT. The program writes the row of the query on one line, with one space between the values.

use sqlx::postgres::PgConnectOptions;
use sqlx::{Connection, PgConnection};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), sqlx::Error> {
    let mut conn = PgConnection::connect_with(&PgConnectOptions::new()).await?;
    let (user, ssl, version): (String, bool, String) = sqlx::query_as(
        "SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()",
    )
    .fetch_one(&mut conn)
    .await?;
    println!("{user} {} {version}", if ssl { "t" } else { "f" });
    conn.close().await
}
