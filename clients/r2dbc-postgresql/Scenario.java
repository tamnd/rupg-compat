// The trace scenario of spec/21 section 21.4.2, with the R2DBC driver for PostgreSQL. Each driver runs the same 10 steps.
// The scenario waits for each step with block(), so the order of the messages is the same each time.
import io.r2dbc.postgresql.PostgresqlConnectionConfiguration;
import io.r2dbc.postgresql.PostgresqlConnectionFactory;
import io.r2dbc.postgresql.api.PostgresqlConnection;
import io.r2dbc.spi.R2dbcException;
import io.r2dbc.spi.Result;
import java.math.BigDecimal;

public class Scenario {
    static PostgresqlConnection c;

    static void run(String sql, Object... params) {
        var s = c.createStatement(sql);
        for (int i = 0; i < params.length; i++) {
            s.bind(i, params[i]);
        }
        s.execute().flatMap(r -> r.map((row, meta) -> {
            for (int i = 0; i < meta.getColumnMetadatas().size(); i++) {
                row.get(i);
            }
            return 1;
        })).blockLast();
    }

    static void sql(String sql) {
        c.createStatement(sql).execute().flatMap(Result::getRowsUpdated).blockLast();
    }

    static void want22012(String sql) {
        try {
            sql(sql);
            throw new IllegalStateException("want division by zero");
        } catch (R2dbcException e) {
            if (!"22012".equals(e.getSqlState())) {
                throw e;
            }
        }
    }

    public static void main(String[] args) {
        var env = System.getenv();
        var config = PostgresqlConnectionConfiguration.builder()
                .host(env.get("PGHOST"))
                .port(Integer.parseInt(env.get("PGPORT")))
                .username(env.get("PGUSER"))
                .password(env.get("PGPASSWORD"))
                .database(env.get("PGDATABASE"))
                .build();
        c = new PostgresqlConnectionFactory(config).create().block();
        // 1. A simple query.
        sql("SELECT 1");
        // 2. A query with parameters.
        run("SELECT $1::int4 + 1, $2::text", 41, "abc");
        // 3. The common types.
        run("SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true, 'x'::text, '\\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date, ARRAY[1, 2, 3]::int4[], '{\"a\": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid, NULL::text");
        // 4. A table.
        sql("CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2))");
        // 5. A transaction that commits.
        c.beginTransaction().block();
        String[] names = {"one", "two", "three"};
        for (int i = 0; i < 3; i++) {
            run("INSERT INTO compat_items VALUES ($1, $2, $3)", i + 1, names[i], new BigDecimal(i + 1 + ".50"));
        }
        c.commitTransaction().block();
        // 6. A transaction that rolls back.
        c.beginTransaction().block();
        run("INSERT INTO compat_items VALUES ($1, $2, $3)", 4, "four", new BigDecimal("4.50"));
        c.rollbackTransaction().block();
        // 7. A statement with parameters, run six times. The driver keeps the named statement in its cache.
        for (int i = 0; i < 6; i++) {
            run("SELECT id, name, price FROM compat_items WHERE id >= $1 ORDER BY id", 1);
        }
        // 8. An update.
        run("UPDATE compat_items SET price = price * 2 WHERE id = $1", 2);
        // 9. An error, then an error in a transaction.
        want22012("SELECT 1 / 0");
        c.beginTransaction().block();
        want22012("SELECT 1 / 0");
        c.rollbackTransaction().block();
        // 10. The end.
        sql("DROP TABLE compat_items");
        c.close().block();
    }
}
