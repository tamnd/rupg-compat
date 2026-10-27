// The connect program of the connect suite, with Npgsql. Program.cs runs it with the argument connect.
// Npgsql does not read the PG variables of libpq, so the program gives them in the connection string. The program writes the row of the query on one line, with one space between the values.
using Npgsql;

static class Connect
{
    static string Env(string name) => Environment.GetEnvironmentVariable(name) ?? throw new InvalidOperationException(name);

    public static void Run()
    {
        var cs = new NpgsqlConnectionStringBuilder
        {
            Host = Env("PGHOST"),
            Port = int.Parse(Env("PGPORT")),
            Database = Env("PGDATABASE"),
            Username = Env("PGUSER"),
            Password = Env("PGPASSWORD"),
            SslMode = SslMode.VerifyFull,
            RootCertificate = Env("PGSSLROOTCERT"),
            ChannelBinding = ChannelBinding.Require,
            Pooling = false,
        }.ConnectionString;
        using var c = new NpgsqlConnection(cs);
        c.Open();
        using var cmd = new NpgsqlCommand("SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()", c);
        using var r = cmd.ExecuteReader();
        r.Read();
        Console.WriteLine($"{r.GetString(0)} {(r.GetBoolean(1) ? "t" : "f")} {r.GetString(2)}");
    }
}
