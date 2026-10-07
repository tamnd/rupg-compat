// The trace scenario of spec/21 section 21.4.2, with node-postgres. Each client runs the same steps.
// node-postgres reads PGHOST, PGPORT, PGUSER, PGPASSWORD and PGDATABASE.
import pg from "pg";

const client = new pg.Client();
await client.connect();

async function divisionByZero(sql) {
  try {
    await client.query(sql);
  } catch (e) {
    if (e.code === "22012") return;
    throw e;
  }
  throw new Error("want 22012");
}

// 1. A simple query.
await client.query("SELECT 1");
// 2. A query with parameters.
await client.query("SELECT $1::int4 + 1, $2::text", [41, "abc"]);
// 3. The common types.
await client.query(
  "SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true," +
    " 'x'::text, '\\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date," +
    " ARRAY[1, 2, 3]::int4[], '{\"a\": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid," +
    " NULL::text",
);
// 4. A table.
await client.query("CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2))");
// 5. A transaction that commits.
await client.query("BEGIN");
for (const row of [[1, "one", "1.50"], [2, "two", "2.50"], [3, "three", "3.50"]]) {
  await client.query("INSERT INTO compat_items VALUES ($1, $2, $3)", row);
}
await client.query("COMMIT");
// 6. A transaction that rolls back.
await client.query("BEGIN");
await client.query("INSERT INTO compat_items VALUES ($1, $2, $3)", [4, "four", "4.50"]);
await client.query("ROLLBACK");
// 7. The same statement, run six times. A name makes node-postgres prepare it once.
for (let i = 0; i < 6; i++) {
  await client.query({ name: "compat_select", text: "SELECT id, name, price FROM compat_items WHERE id >= $1 ORDER BY id", values: [1] });
}
// 8. An update.
await client.query("UPDATE compat_items SET price = price * 2 WHERE id = $1", [2]);
// 9. An error, then an error in a transaction.
await divisionByZero("SELECT 1 / 0");
await client.query("BEGIN");
await divisionByZero("SELECT 1 / 0");
await client.query("ROLLBACK");
// 10. The end.
await client.query("DROP TABLE compat_items");
await client.end();
