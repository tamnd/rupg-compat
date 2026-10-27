// The connect program of the connect suite, with pgjdbc.
// pgjdbc does not read the PG variables of libpq, so the program gives them in the URL and the properties. The program writes the row of the query on one line, with one space between the values.
import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.ResultSet;
import java.sql.Statement;
import java.util.Properties;

public class Connect {
    public static void main(String[] args) throws Exception {
        var env = System.getenv();
        var url = "jdbc:postgresql://" + env.get("PGHOST") + ":" + env.get("PGPORT") + "/" + env.get("PGDATABASE");
        var props = new Properties();
        props.setProperty("user", env.get("PGUSER"));
        props.setProperty("password", env.get("PGPASSWORD"));
        props.setProperty("sslmode", env.get("PGSSLMODE"));
        props.setProperty("sslrootcert", env.get("PGSSLROOTCERT"));
        props.setProperty("channelBinding", "require");
        try (Connection c = DriverManager.getConnection(url, props);
                Statement s = c.createStatement();
                ResultSet r = s.executeQuery("SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()")) {
            r.next();
            System.out.println(r.getString(1) + " " + (r.getBoolean(2) ? "t" : "f") + " " + r.getString(3));
        }
    }
}
