//! The trace scenario of spec/21 section 21.4.2, with tokio-postgres. Each client runs the same steps.

use tokio_postgres::error::SqlState;
use tokio_postgres::{Client, Config, Error, NoTls};

fn env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("{key} is not set"))
}

/// tokio-postgres does not read the libpq environment, so the scenario reads it.
fn config() -> Config {
    let mut c = Config::new();
    c.host(env("PGHOST"))
        .port(env("PGPORT").parse().expect("PGPORT is a number"))
        .user(env("PGUSER"))
        .password(env("PGPASSWORD"))
        .dbname(env("PGDATABASE"));
    c
}

fn division_by_zero(e: Error) {
    assert_eq!(e.code(), Some(&SqlState::DIVISION_BY_ZERO), "{e}");
}

const TYPES: &str = "SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true, \
    'x'::text, '\\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date, \
    ARRAY[1, 2, 3]::int4[], '{\"a\": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid, NULL::text";

// The driver sends a Rust string only as text, so the price goes through text to numeric.
const INSERT: &str = "INSERT INTO compat_items VALUES ($1, $2, $3::text::numeric)";

async fn scenario(client: &mut Client) -> Result<(), Error> {
    // 1. A simple query.
    client.query("SELECT 1", &[]).await?;
    // 2. A query with parameters.
    client.query("SELECT $1::int4 + 1, $2::text", &[&41i32, &"abc"]).await?;
    // 3. The common types.
    client.query(TYPES, &[]).await?;
    // 4. A table.
    client
        .execute("CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2))", &[])
        .await?;
    // 5. A transaction that commits.
    let tx = client.transaction().await?;
    for (id, name, price) in [(1i32, "one", "1.50"), (2, "two", "2.50"), (3, "three", "3.50")] {
        tx.execute(INSERT, &[&id, &name, &price]).await?;
    }
    tx.commit().await?;
    // 6. A transaction that rolls back.
    let tx = client.transaction().await?;
    tx.execute(INSERT, &[&4i32, &"four", &"4.50"]).await?;
    tx.rollback().await?;
    // 7. The same statement, run six times.
    for _ in 0..6 {
        client.query("SELECT id, name, price FROM compat_items WHERE id >= $1 ORDER BY id", &[&1i32]).await?;
    }
    // 8. An update.
    client.execute("UPDATE compat_items SET price = price * 2 WHERE id = $1", &[&2i32]).await?;
    // 9. An error, then an error in a transaction.
    division_by_zero(client.query("SELECT 1 / 0", &[]).await.unwrap_err());
    let tx = client.transaction().await?;
    division_by_zero(tx.query("SELECT 1 / 0", &[]).await.unwrap_err());
    tx.rollback().await?;
    // 10. The end.
    client.execute("DROP TABLE compat_items", &[]).await?;
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let (mut client, connection) = config().connect(NoTls).await?;
    let task = tokio::spawn(connection);
    scenario(&mut client).await?;
    drop(client);
    task.await.expect("the connection task")?;
    Ok(())
}
