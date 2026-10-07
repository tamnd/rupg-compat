# The trace scenario of spec/21 section 21.4.2, with Postgrex. Each client runs the same steps.
# Postgrex takes the host, the port, the user, the password and the database from the PG variables of the environment.
{:ok, conn} = Postgrex.start_link(ssl: false, pool_size: 1)
# 1. A simple query.
Postgrex.query!(conn, "SELECT 1", [])
# 2. A query with parameters.
Postgrex.query!(conn, "SELECT $1::int4 + 1, $2::text", [41, "abc"])
# 3. The common types.
Postgrex.query!(
  conn,
  "SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true, " <>
    "'x'::text, '\\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date, " <>
    "ARRAY[1, 2, 3]::int4[], '{\"a\": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid, " <>
    "NULL::text",
  []
)
# 4. A table.
Postgrex.query!(conn, "CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2))", [])
# 5. A transaction that commits.
{:ok, _} =
  Postgrex.transaction(conn, fn c ->
    for {id, name, price} <- [{1, "one", "1.50"}, {2, "two", "2.50"}, {3, "three", "3.50"}] do
      Postgrex.query!(c, "INSERT INTO compat_items VALUES ($1, $2, $3)", [id, name, Decimal.new(price)])
    end
  end)
# 6. A transaction that rolls back.
{:error, :rollback} =
  Postgrex.transaction(conn, fn c ->
    Postgrex.query!(c, "INSERT INTO compat_items VALUES ($1, $2, $3)", [4, "four", Decimal.new("4.50")])
    Postgrex.rollback(c, :rollback)
  end)
# 7. The same statement, prepared once and run six times.
query = Postgrex.prepare!(conn, "items", "SELECT id, name, price FROM compat_items WHERE id >= $1 ORDER BY id")
for _ <- 1..6, do: Postgrex.execute!(conn, query, [1])
# 8. An update.
Postgrex.query!(conn, "UPDATE compat_items SET price = price * 2 WHERE id = $1", [2])
# 9. An error, then an error in a transaction.
{:error, %Postgrex.Error{postgres: %{code: :division_by_zero}}} = Postgrex.query(conn, "SELECT 1 / 0", [])
{:error, :division_by_zero} =
  Postgrex.transaction(conn, fn c ->
    {:error, %Postgrex.Error{postgres: %{code: code}}} = Postgrex.query(c, "SELECT 1 / 0", [])
    Postgrex.rollback(c, code)
  end)
# 10. The end.
Postgrex.query!(conn, "DROP TABLE compat_items", [])
GenServer.stop(conn)
