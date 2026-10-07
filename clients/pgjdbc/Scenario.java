// The trace scenario of spec/21 section 21.4.2, with pgjdbc. Each driver runs the same 10 steps.
// pgjdbc uses a named server statement from the fifth run of a PreparedStatement (prepareThreshold), so step 7 runs it six times.
import java.math.BigDecimal;
import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.sql.Statement;
import java.util.Properties;

public class Scenario {
    public static void main(String[] args) throws Exception {
        var env = System.getenv();
        var url = "jdbc:postgresql://" + env.get("PGHOST") + ":" + env.get("PGPORT") + "/" + env.get("PGDATABASE");
        var props = new Properties();
        props.setProperty("user", env.get("PGUSER"));
        props.setProperty("password", env.get("PGPASSWORD"));
        props.setProperty("sslmode", "disable");
        try (Connection c = DriverManager.getConnection(url, props)) {
            // 1. A simple query.
            try (Statement s = c.createStatement()) {
                s.execute("SELECT 1");
            }
            // 2. A query with parameters.
            try (PreparedStatement p = c.prepareStatement("SELECT ?::int4 + 1, ?::text")) {
                p.setInt(1, 41);
                p.setString(2, "abc");
                drain(p.executeQuery());
            }
            // 3. The common types.
            try (Statement s = c.createStatement()) {
                drain(s.executeQuery("SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true, 'x'::text, '\\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date, ARRAY[1, 2, 3]::int4[], '{\"a\": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid, NULL::text"));
                // 4. A table.
                s.execute("CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2))");
            }
            // 5. A transaction that commits.
            c.setAutoCommit(false);
            try (PreparedStatement p = c.prepareStatement("INSERT INTO compat_items VALUES (?, ?, ?)")) {
                String[] names = {"one", "two", "three"};
                for (int i = 0; i < 3; i++) {
                    p.setInt(1, i + 1);
                    p.setString(2, names[i]);
                    p.setBigDecimal(3, new BigDecimal(i + 1 + ".50"));
                    p.executeUpdate();
                }
            }
            c.commit();
            // 6. A transaction that rolls back.
            try (PreparedStatement p = c.prepareStatement("INSERT INTO compat_items VALUES (?, ?, ?)")) {
                p.setInt(1, 4);
                p.setString(2, "four");
                p.setBigDecimal(3, new BigDecimal("4.50"));
                p.executeUpdate();
            }
            c.rollback();
            c.setAutoCommit(true);
            // 7. A prepared statement, run six times.
            try (PreparedStatement p = c.prepareStatement("SELECT id, name, price FROM compat_items WHERE id >= ? ORDER BY id")) {
                for (int i = 0; i < 6; i++) {
                    p.setInt(1, 1);
                    drain(p.executeQuery());
                }
            }
            // 8. An update.
            try (PreparedStatement p = c.prepareStatement("UPDATE compat_items SET price = price * 2 WHERE id = ?")) {
                p.setInt(1, 2);
                p.executeUpdate();
            }
            // 9. An error, then an error in a transaction.
            try (Statement s = c.createStatement()) {
                want22012(s);
                c.setAutoCommit(false);
                want22012(s);
                c.rollback();
                c.setAutoCommit(true);
                // 10. The end.
                s.execute("DROP TABLE compat_items");
            }
        }
    }

    static void drain(ResultSet r) throws SQLException {
        try (r) {
            while (r.next()) {
                for (int i = 1; i <= r.getMetaData().getColumnCount(); i++) {
                    r.getObject(i);
                }
            }
        }
    }

    static void want22012(Statement s) throws SQLException {
        try {
            s.execute("SELECT 1 / 0");
            throw new IllegalStateException("want division by zero");
        } catch (SQLException e) {
            if (!"22012".equals(e.getSQLState())) {
                throw e;
            }
        }
    }
}
