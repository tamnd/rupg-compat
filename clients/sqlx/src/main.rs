//! The trace scenario of spec/21 section 21.4.2, with sqlx. Each client runs the same steps.

use sqlx::postgres::PgConnectOptions;
use sqlx::{Connection, Executor, PgConnection};

fn division_by_zero(e: sqlx::Error) {
    let code = e.as_database_error().and_then(|d| d.code().map(|c| c.into_owned()));
    assert_eq!(code.as_deref(), Some("22012"), "{e}");
}

const TYPES: &str = "SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true, \
    'x'::text, '\\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date, \
    ARRAY[1, 2, 3]::int4[], '{\"a\": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid, NULL::text";

// The scenario binds the price as a string, so it goes through text to numeric.
const INSERT: &str = "INSERT INTO compat_items VALUES ($1, $2, $3::text::numeric)";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), sqlx::Error> {
    // `PgConnectOptions::new` reads PGHOST, PGPORT, PGUSER, PGPASSWORD, PGDATABASE and PGSSLMODE.
    let mut conn = PgConnection::connect_with(&PgConnectOptions::new()).await?;
    // 1. A simple query.
    sqlx::query("SELECT 1").fetch_all(&mut conn).await?;
    // 2. A query with parameters.
    sqlx::query("SELECT $1::int4 + 1, $2::text").bind(41i32).bind("abc").fetch_all(&mut conn).await?;
    // 3. The common types.
    sqlx::query(TYPES).fetch_all(&mut conn).await?;
    // 4. A table.
    conn.execute("CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2))").await?;
    // 5. A transaction that commits.
    let mut tx = conn.begin().await?;
    for (id, name, price) in [(1i32, "one", "1.50"), (2, "two", "2.50"), (3, "three", "3.50")] {
        sqlx::query(INSERT).bind(id).bind(name).bind(price).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    // 6. A transaction that rolls back.
    let mut tx = conn.begin().await?;
    sqlx::query(INSERT).bind(4i32).bind("four").bind("4.50").execute(&mut *tx).await?;
    tx.rollback().await?;
    // 7. The same statement, run six times.
    for _ in 0..6 {
        sqlx::query("SELECT id, name, price FROM compat_items WHERE id >= $1 ORDER BY id")
            .bind(1i32)
            .fetch_all(&mut conn)
            .await?;
    }
    // 8. An update.
    sqlx::query("UPDATE compat_items SET price = price * 2 WHERE id = $1").bind(2i32).execute(&mut conn).await?;
    // 9. An error, then an error in a transaction.
    division_by_zero(sqlx::query("SELECT 1 / 0").fetch_all(&mut conn).await.unwrap_err());
    let mut tx = conn.begin().await?;
    division_by_zero(sqlx::query("SELECT 1 / 0").fetch_all(&mut *tx).await.unwrap_err());
    tx.rollback().await?;
    // 10. The end.
    conn.execute("DROP TABLE compat_items").await?;
    conn.close().await
}
