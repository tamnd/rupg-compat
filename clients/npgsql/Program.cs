// The trace scenario of spec/21 section 21.4.2, with Npgsql. Each driver runs the same 10 steps.
// Npgsql loads the types of the database when it opens the connection. Step 7 prepares the statement on the server with Prepare and runs it six times.
// The connection does not use the pool, so Close ends the session with Terminate.
using Npgsql;

static string Env(string name) => Environment.GetEnvironmentVariable(name) ?? throw new InvalidOperationException(name);

var cs = new NpgsqlConnectionStringBuilder
{
    Host = Env("PGHOST"),
    Port = int.Parse(Env("PGPORT")),
    Database = Env("PGDATABASE"),
    Username = Env("PGUSER"),
    Password = Env("PGPASSWORD"),
    SslMode = SslMode.Disable,
    Pooling = false,
}.ConnectionString;
using var c = new NpgsqlConnection(cs);
c.Open();

// 1. A simple query.
Exec("SELECT 1");
// 2. A query with parameters.
using (var cmd = new NpgsqlCommand("SELECT $1::int4 + 1, $2::text", c))
{
    cmd.Parameters.Add(new NpgsqlParameter { Value = 41 });
    cmd.Parameters.Add(new NpgsqlParameter { Value = "abc" });
    Drain(cmd);
}
// 3. The common types.
using (var cmd = new NpgsqlCommand("SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true, 'x'::text, '\\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date, ARRAY[1, 2, 3]::int4[], '{\"a\": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid, NULL::text", c))
{
    Drain(cmd);
}
// 4. A table.
Exec("CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2))");
// 5. A transaction that commits.
using (var tx = c.BeginTransaction())
{
    string[] names = ["one", "two", "three"];
    for (var i = 0; i < 3; i++)
    {
        Insert(i + 1, names[i], i + 1.5m, tx);
    }
    tx.Commit();
}
// 6. A transaction that rolls back.
using (var tx = c.BeginTransaction())
{
    Insert(4, "four", 4.5m, tx);
    tx.Rollback();
}
// 7. A prepared statement, run six times.
using (var cmd = new NpgsqlCommand("SELECT id, name, price FROM compat_items WHERE id >= $1 ORDER BY id", c))
{
    cmd.Parameters.Add(new NpgsqlParameter { Value = 1 });
    cmd.Prepare();
    for (var i = 0; i < 6; i++)
    {
        Drain(cmd);
    }
    cmd.Unprepare();
}
// 8. An update.
using (var cmd = new NpgsqlCommand("UPDATE compat_items SET price = price * 2 WHERE id = $1", c))
{
    cmd.Parameters.Add(new NpgsqlParameter { Value = 2 });
    cmd.ExecuteNonQuery();
}
// 9. An error, then an error in a transaction.
Want22012(null);
using (var tx = c.BeginTransaction())
{
    Want22012(tx);
    tx.Rollback();
}
// 10. The end.
Exec("DROP TABLE compat_items");
c.Close();

void Exec(string sql)
{
    using var cmd = new NpgsqlCommand(sql, c);
    cmd.ExecuteNonQuery();
}

void Insert(int id, string name, decimal price, NpgsqlTransaction tx)
{
    using var cmd = new NpgsqlCommand("INSERT INTO compat_items VALUES ($1, $2, $3)", c, tx);
    cmd.Parameters.Add(new NpgsqlParameter { Value = id });
    cmd.Parameters.Add(new NpgsqlParameter { Value = name });
    cmd.Parameters.Add(new NpgsqlParameter { Value = price });
    cmd.ExecuteNonQuery();
}

static void Drain(NpgsqlCommand cmd)
{
    using var r = cmd.ExecuteReader();
    while (r.Read())
    {
        for (var i = 0; i < r.FieldCount; i++)
        {
            r.GetValue(i);
        }
    }
}

void Want22012(NpgsqlTransaction? tx)
{
    try
    {
        using var cmd = new NpgsqlCommand("SELECT 1 / 0", c, tx);
        cmd.ExecuteNonQuery();
        throw new InvalidOperationException("want division by zero");
    }
    catch (PostgresException e) when (e.SqlState == "22012")
    {
    }
}
