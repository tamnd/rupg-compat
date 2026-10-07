# The trace scenario of spec/21 section 21.4.2, with the Ruby pg gem. Each client runs the same steps.
require "pg"

conn = PG.connect
# 1. A simple query.
conn.exec("SELECT 1").values
# 2. A query with parameters.
conn.exec_params("SELECT $1::int4 + 1, $2::text", [41, "abc"]).values
# 3. The common types.
conn.exec(
  "SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true, " \
  "'x'::text, '\\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date, " \
  "ARRAY[1, 2, 3]::int4[], '{\"a\": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid, " \
  "NULL::text"
).values
# 4. A table.
conn.exec("CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2))")
# 5. A transaction that commits.
conn.transaction do |c|
  [[1, "one", "1.50"], [2, "two", "2.50"], [3, "three", "3.50"]].each do |row|
    c.exec_params("INSERT INTO compat_items VALUES ($1, $2, $3)", row)
  end
end
# 6. A transaction that rolls back.
conn.exec("BEGIN")
conn.exec_params("INSERT INTO compat_items VALUES ($1, $2, $3)", [4, "four", "4.50"])
conn.exec("ROLLBACK")
# 7. The same statement, prepared once and run six times. The rows come in binary format.
conn.prepare("items", "SELECT id, name, price FROM compat_items WHERE id >= $1 ORDER BY id")
6.times { conn.exec_prepared("items", [1], 1).values }
# 8. An update.
conn.exec_params("UPDATE compat_items SET price = price * 2 WHERE id = $1", [2])
# 9. An error, then an error in a transaction.
begin
  conn.exec("SELECT 1 / 0")
rescue PG::DivisionByZero
end
conn.exec("BEGIN")
begin
  conn.exec("SELECT 1 / 0")
rescue PG::DivisionByZero
end
conn.exec("ROLLBACK")
# 10. The end.
conn.exec("DROP TABLE compat_items")
conn.close
