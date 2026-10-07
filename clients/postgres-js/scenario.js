// The trace scenario of spec/21 section 21.4.2, with postgres.js. Each client runs the same steps.
// postgres.js reads PGHOST, PGPORT, PGUSER, PGPASSWORD and PGDATABASE.
import postgres from "postgres";

const sql = postgres({ max: 1, ssl: false, onnotice: () => {} });

async function divisionByZero(run) {
  try {
    await run;
  } catch (e) {
    if (e.code === "22012") return;
    throw e;
  }
  throw new Error("want 22012");
}

// 1. A simple query.
await sql`SELECT 1`.simple();
// 2. A query with parameters.
await sql`SELECT ${41}::int4 + 1, ${"abc"}::text`;
// 3. The common types.
await sql.unsafe(
  "SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true," +
    " 'x'::text, '\\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date," +
    " ARRAY[1, 2, 3]::int4[], '{\"a\": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid," +
    " NULL::text",
);
// 4. A table.
await sql`CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2))`;
// 5. A transaction that commits.
await sql.begin(async (tx) => {
  for (const [id, name, price] of [[1, "one", "1.50"], [2, "two", "2.50"], [3, "three", "3.50"]]) {
    await tx`INSERT INTO compat_items VALUES (${id}, ${name}, ${price})`;
  }
});
// 6. A transaction that rolls back.
await sql
  .begin(async (tx) => {
    await tx`INSERT INTO compat_items VALUES (${4}, ${"four"}, ${"4.50"})`;
    throw new Error("roll back");
  })
  .catch((e) => {
    if (e.message !== "roll back") throw e;
  });
// 7. The same statement, run six times. postgres.js prepares it once.
for (let i = 0; i < 6; i++) {
  await sql`SELECT id, name, price FROM compat_items WHERE id >= ${1} ORDER BY id`;
}
// 8. An update.
await sql`UPDATE compat_items SET price = price * 2 WHERE id = ${2}`;
// 9. An error, then an error in a transaction.
await divisionByZero(sql`SELECT 1 / 0`);
await sql
  .begin(async (tx) => {
    await divisionByZero(tx`SELECT 1 / 0`);
    throw new Error("roll back");
  })
  .catch((e) => {
    if (e.message !== "roll back") throw e;
  });
// 10. The end.
await sql`DROP TABLE compat_items`;
await sql.end();
