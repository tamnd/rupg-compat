// The gate program of spec/21 section 21.4.2, with pgjdbc, the driver of DBeaver.
// The object browser of a JDBC tool reads the schema with DatabaseMetaData. So the program writes the result of each call of DatabaseMetaData that lists the schemas, the tables, the columns, the keys, the indexes and the types of the schema gate, with each value as pgjdbc gives it as a string.
import java.sql.Connection;
import java.sql.DatabaseMetaData;
import java.sql.DriverManager;
import java.sql.ResultSet;
import java.sql.ResultSetMetaData;
import java.util.ArrayList;
import java.util.Properties;

public class Gate {
    static void print(String call, ResultSet r) throws Exception {
        try (r) {
            ResultSetMetaData m = r.getMetaData();
            var columns = new ArrayList<String>();
            for (int i = 1; i <= m.getColumnCount(); i++) {
                columns.add(m.getColumnLabel(i));
            }
            System.out.println("call: " + call);
            System.out.println("columns: " + String.join(", ", columns));
            while (r.next()) {
                var values = new ArrayList<String>();
                for (int i = 1; i <= m.getColumnCount(); i++) {
                    values.add(String.valueOf(r.getString(i)));
                }
                System.out.println("row: " + String.join(" | ", values));
            }
            System.out.println();
        }
    }

    public static void main(String[] args) throws Exception {
        var env = System.getenv();
        var url = "jdbc:postgresql://" + env.get("PGHOST") + ":" + env.get("PGPORT") + "/" + env.get("PGDATABASE");
        var props = new Properties();
        props.setProperty("user", env.get("PGUSER"));
        props.setProperty("password", env.get("PGPASSWORD"));
        props.setProperty("sslmode", "disable");
        if (env.get("PGOPTIONS") != null) {
            props.setProperty("options", env.get("PGOPTIONS"));
        }
        try (Connection c = DriverManager.getConnection(url, props)) {
            DatabaseMetaData d = c.getMetaData();
            print("getSchemas", d.getSchemas(null, "gate"));
            print("getTables", d.getTables(null, "gate", "%", null));
            print("getColumns", d.getColumns(null, "gate", "%", "%"));
            var tables = new ArrayList<String>();
            try (ResultSet r = d.getTables(null, "gate", "%", new String[] {"TABLE"})) {
                while (r.next()) {
                    tables.add(r.getString("TABLE_NAME"));
                }
            }
            for (String t : tables) {
                print("getPrimaryKeys " + t, d.getPrimaryKeys(null, "gate", t));
                print("getImportedKeys " + t, d.getImportedKeys(null, "gate", t));
                print("getExportedKeys " + t, d.getExportedKeys(null, "gate", t));
                print("getIndexInfo " + t, d.getIndexInfo(null, "gate", t, false, false));
                print("getBestRowIdentifier " + t, d.getBestRowIdentifier(null, "gate", t, DatabaseMetaData.bestRowSession, true));
                print("getTablePrivileges " + t, d.getTablePrivileges(null, "gate", t));
            }
            print("getUDTs", d.getUDTs(null, "gate", "%", null));
            print("getFunctions", d.getFunctions(null, "gate", "%"));
        }
    }
}
