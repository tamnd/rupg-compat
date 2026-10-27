// The connect program of the connect suite, with the R2DBC driver for PostgreSQL.
// The driver does not read the PG variables of libpq, so the program gives them to the builder. The program writes the row of the query on one line, with one space between the values.
import io.r2dbc.postgresql.PostgresqlConnectionConfiguration;
import io.r2dbc.postgresql.PostgresqlConnectionFactory;
import io.r2dbc.postgresql.client.SSLMode;

public class Connect {
    public static void main(String[] args) {
        var env = System.getenv();
        var config = PostgresqlConnectionConfiguration.builder()
                .host(env.get("PGHOST"))
                .port(Integer.parseInt(env.get("PGPORT")))
                .username(env.get("PGUSER"))
                .password(env.get("PGPASSWORD"))
                .database(env.get("PGDATABASE"))
                .sslMode(SSLMode.VERIFY_FULL)
                .sslRootCert(env.get("PGSSLROOTCERT"))
                .build();
        var c = new PostgresqlConnectionFactory(config).create().block();
        var line = c.createStatement("SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()")
                .execute()
                .flatMap(r -> r.map((row, meta) -> row.get(0, String.class) + " " + (row.get(1, Boolean.class) ? "t" : "f") + " " + row.get(2, String.class)))
                .blockLast();
        System.out.println(line);
        c.close().block();
    }
}
