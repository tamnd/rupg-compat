// The trace scenario for DBeaver 26.2.3. It sends the SQL of the PostgreSQL plugin of DBeaver in the order that DBeaver sends it: connect, open the schema compat_dbeaver in the navigator, show the DDL of a table and read the first 200 rows of the table.
// The SQL text is the text that the plugin builds for a server of version 19 with the default settings. DBeaver uses a Main and a Metadata connection, but the scenario uses one connection.
import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Properties;
import java.util.Set;

public class Scenario {
    // PostgreSchema.TableCache.prepareChildrenStatement builds this text for one table.
    static final String COLUMNS = "SELECT c.relname,a.*,pg_catalog.pg_get_expr(ad.adbin, ad.adrelid, true) as def_value,dsc.description,dep.objid"
        + "\nFROM pg_catalog.pg_attribute a"
        + "\nINNER JOIN pg_catalog.pg_class c ON (a.attrelid=c.oid)"
        + "\nLEFT OUTER JOIN pg_catalog.pg_attrdef ad ON (a.attrelid=ad.adrelid AND a.attnum = ad.adnum)"
        + "\nLEFT OUTER JOIN pg_catalog.pg_description dsc ON (c.oid=dsc.objoid AND a.attnum = dsc.objsubid)"
        + "\nLEFT OUTER JOIN pg_depend dep on dep.refobjid = a.attrelid AND dep.deptype = 'i' and dep.refobjsubid = a.attnum and dep.classid = dep.refclassid"
        + "\nWHERE NOT a.attisdropped AND c.relkind not in ('i','I','c') AND c.oid=?"
        + "\nORDER BY a.attnum";
    // PostgreSchema.ConstraintCache.prepareObjectsStatement builds this text for one table.
    static final String CONSTRAINTS = "SELECT c.oid,c.*,t.relname as tabrelname,rt.relnamespace as refnamespace,d.description, case when c.contype='c' then \"substring\"(pg_get_constraintdef(c.oid), 7) else null end consrc_copy"
        + "\nFROM pg_catalog.pg_constraint c"
        + "\nINNER JOIN pg_catalog.pg_class t ON t.oid=c.conrelid"
        + "\nLEFT OUTER JOIN pg_catalog.pg_class rt ON rt.oid=c.confrelid"
        + "\nLEFT OUTER JOIN pg_catalog.pg_description d ON d.objoid=c.oid AND d.objsubid=0 AND d.classoid='pg_constraint'::regclass"
        + "\nWHERE c.conrelid=?"
        + "\nORDER BY c.oid";
    // PostgreSchema.IndexCache.prepareObjectsStatement builds this text for one table.
    static final String INDEXES = "SELECT i.*,i.indkey as keys,c.relname,c.relnamespace,c.relam,c.reltablespace,tc.relname as tabrelname,dsc.description"
        + ",pg_catalog.pg_get_expr(i.indpred, i.indrelid) as pred_expr"
        + ",pg_catalog.pg_get_expr(i.indexprs, i.indrelid, true) as expr"
        + ",pg_catalog.pg_relation_size(i.indexrelid) as index_rel_size"
        + ",pg_catalog.pg_stat_get_numscans(i.indexrelid) as index_num_scans"
        + """
            \sFROM pg_catalog.pg_index i
            INNER JOIN pg_catalog.pg_class c ON c.oid=i.indexrelid
            INNER JOIN pg_catalog.pg_class tc ON tc.oid=i.indrelid
            LEFT OUTER JOIN pg_catalog.pg_description dsc ON i.indexrelid=dsc.objoid
            WHERE\s"""
        + " i.indrelid=?"
        + " ORDER BY tabrelname, c.relname";
    // PostgreDatabase.cacheDataTypes and PostgreDataTypeCache.prepareObjectsStatement start with this text.
    static final String DATA_TYPES = "SELECT t.oid,t.*,c.relkind,format_type(nullif(t.typbasetype, 0), t.typtypmod) as base_type_name, d.description"
        + "\nFROM pg_catalog.pg_type t"
        + "\nLEFT OUTER JOIN pg_catalog.pg_type et ON et.oid=t.typelem "
        + "\nLEFT OUTER JOIN pg_catalog.pg_class c ON c.oid=t.typrelid"
        + "\nLEFT OUTER JOIN pg_catalog.pg_description d ON t.oid=d.objoid"
        + "\nWHERE t.typname IS NOT NULL"
        + "\nAND (c.relkind IS NULL OR c.relkind = 'c') AND (et.typcategory IS NULL OR et.typcategory <> 'C')";

    public static void main(String[] args) throws Exception {
        var env = System.getenv();
        var url = "jdbc:postgresql://" + env.get("PGHOST") + ":" + env.get("PGPORT") + "/" + env.get("PGDATABASE");
        var props = new Properties();
        props.setProperty("user", env.get("PGUSER"));
        props.setProperty("password", env.get("PGPASSWORD"));
        props.setProperty("sslmode", "disable");
        var db = env.get("PGDATABASE");
        try (Connection c = DriverManager.getConnection(url, props)) {
            // PostgreDataSource.openConnection: DBeaver sets the name from DBUtils.getClientApplicationName. pgjdbc sends it with SET application_name.
            c.setClientInfo("ApplicationName", "DBeaver 26.2.3 - Main <" + db + ">");

            // 1. Connect.
            // PostgreExecutionContext.refreshDefaults: the search path.
            query(c, "SELECT reset_val FROM pg_settings WHERE name = 'search_path'");
            // PostgreExecutionContext.refreshDefaults: the current schema and user.
            query(c, "SELECT current_schema(),session_user");
            // PostgreDatabase.SchemaCache.prepareLookupStatement, from PostgreExecutionContext.setActiveSchema: all schemas, with no filter.
            var schemas = query(c, "SELECT n.oid,n.*,d.description FROM pg_catalog.pg_namespace n\n"
                + "LEFT OUTER JOIN pg_catalog.pg_description d ON d.objoid=n.oid AND d.objsubid=0 AND d.classoid='pg_namespace'::regclass\n"
                + " ORDER BY nspname");
            long schema = oid(schemas, "nspname", "compat_dbeaver", "oid");
            // PostgreDatabase.readDatabaseInfo: the current database. This is the database list when "Show all databases" is off.
            query(c, "SELECT db.oid,db.* FROM pg_catalog.pg_database db WHERE datname=?", db);
            // PostgreDataSource.SettingCache.prepareLookupStatement, from PostgreDataSource.initializeRemoteInstance: one setting.
            query(c, "select * from pg_catalog.pg_settings where name=?", "standard_conforming_strings");
            // PostgreDataSource.initialize: the server version.
            query(c, "SELECT version()");
            // PostgreUtils.isMetaObjectExists, from PostgreDataSource.initialize: a check for pg_enum on a server of version 12 or later.
            query(c, "SELECT * FROM pg_catalog.pg_enum WHERE 1<>1 LIMIT 1");
            // PostgreUtils.isMetaObjectExists, from PostgreDataSource.initialize: a check for pg_class.reltype.
            query(c, "SELECT reltype FROM pg_catalog.pg_class WHERE 1<>1 LIMIT 1");
            // PostgreDatabase.cacheDataTypes: the data types of all schemas, without table types and their arrays.
            query(c, DATA_TYPES);
            // PostgreDatabase.EnumValueCache.prepareObjectsStatement, from PostgreDatabase.initEnumTypesCache: all enum values.
            query(c, "SELECT * FROM pg_catalog.pg_enum");
            // PostgreDatabase.RoleCache.prepareObjectsStatement: the roles, with their comments.
            query(c, "SELECT a.oid,a.*,pd.description FROM pg_catalog.pg_roles a \nleft join pg_catalog.pg_shdescription pd on a.oid = pd.objoid\nORDER BY a.rolname");
            // PostgreDataSource.SettingCache.prepareLookupStatement, from PostgreDatabase.getSettings: all settings.
            query(c, "select * from pg_catalog.pg_settings");

            // 2. The navigator opens compat_dbeaver. DBeaver loads the columns and the constraints of a table before its indexes, because PostgreIndex looks up the constraint of a unique index.
            // PostgreSchema.TableCache.prepareLookupStatement: the tables, views and sequences of the schema.
            var relations = query(c, "SELECT c.oid,c.*,d.description,pg_catalog.pg_get_expr(c.relpartbound, c.oid) as partition_expr,  pg_catalog.pg_get_partkeydef(c.oid) as partition_key "
                + "\nFROM pg_catalog.pg_class c\n"
                + "LEFT OUTER JOIN pg_catalog.pg_description d ON d.objoid=c.oid AND d.objsubid=0 AND d.classoid='pg_class'::regclass\n"
                + "WHERE c.relnamespace=? AND c.relkind not in ('i','I','c')", schema);
            long items = oid(relations, "relname", "items", "oid");
            long orders = oid(relations, "relname", "orders", "oid");
            long view = oid(relations, "relname", "item_totals", "oid");
            // PostgreSchema.TableCache.prepareChildrenStatement: the columns of items.
            query(c, COLUMNS, items);
            // PostgreSchema.ConstraintCache.prepareObjectsStatement: the constraints of items.
            var itemConstraints = query(c, CONSTRAINTS, items);
            // PostgreSchema.IndexCache.prepareObjectsStatement: the indexes of items.
            var itemIndexes = query(c, INDEXES, items);
            // PostgreDatabase.AccessMethodCache.prepareObjectsStatement, from PostgreIndex.getAccessMethod: the access methods, for the first index.
            query(c, "SELECT am.oid,am.* FROM pg_catalog.pg_am am \nORDER BY am.oid");
            for (var index : itemIndexes) {
                if (index.get("expr") == null) {
                    continue;
                }
                var keys = String.valueOf(index.get("keys")).trim().split(" +");
                for (int i = 0; i < keys.length; i++) {
                    if (keys[i].equals("0")) {
                        // PostgreSchema.IndexCache.fetchObjectRow: the expression of an index column that is not a table column.
                        query(c, "select pg_catalog.pg_get_indexdef(?, ?, true)", ((Number) index.get("indexrelid")).longValue(), i + 1);
                    }
                }
            }
            // PostgreSchema.TableCache.prepareChildrenStatement: the columns of orders.
            query(c, COLUMNS, orders);
            // PostgreSchema.ConstraintCache.prepareObjectsStatement: the constraints of orders. The foreign key uses the columns of items from the cache.
            query(c, CONSTRAINTS, orders);
            // PostgreSchema.IndexCache.prepareObjectsStatement: the indexes of orders.
            query(c, INDEXES, orders);
            // PostgreSequence.loadAdditionalInfo: the sequence, from pg_sequences on a server of version 10 or later.
            query(c, "SELECT * from pg_catalog.pg_sequences WHERE schemaname=? AND sequencename=?", "compat_dbeaver", "item_ids");
            // PostgreSchema.TableCache.prepareChildrenStatement: the columns of the view.
            query(c, COLUMNS, view);
            // PostgreViewBase.fetchViewQueryResultIfItNull: the view definition. DBeaver puts the oid in the text.
            query(c, "SELECT pg_get_viewdef(" + view + ", true)");
            // PostgreDataTypeCache.prepareObjectsStatement: the data types of the schema. The schema has no data types from the full load, so DBeaver reads them again.
            query(c, DATA_TYPES + "\nAND t.typnamespace=? ", schema);
            // PostgreSchema.ProceduresCache.prepareLookupStatement: the functions and procedures of the schema.
            query(c, "SELECT p.oid as poid,p.*,pg_catalog.pg_get_expr(p.proargdefaults, 0) as arg_defaults,d.description\n"
                + "FROM pg_catalog.pg_proc p\n"
                + "LEFT OUTER JOIN pg_catalog.pg_description d ON d.objoid=p.oid and d.classoid='pg_proc'::regclass  AND d.objsubid = 0"
                + "\nWHERE p.pronamespace=?"
                + "\nORDER BY p.proname", schema);

            // 3. The DDL of items, from DBStructUtils.generateTableDDL and PostgreTableManagerBase. The columns, constraints and indexes come from the cache.
            // PostgreDatabase.CollationCache.prepareObjectsStatement, from PostgreTableColumnManager.PostgreCollateModifier: the collations, for the first text column.
            query(c, "SELECT c.oid,c.* FROM pg_catalog.pg_collation c \nORDER BY c.oid");
            for (var constraint : itemConstraints) {
                if (Set.of("p", "u", "c", "x").contains(String.valueOf(constraint.get("contype")))) {
                    // PostgreTableConstraintBase.getObjectDefinitionText, from PostgreConstraintManager.getNestedDeclaration: the text of a constraint.
                    query(c, "SELECT pg_catalog.pg_get_constraintdef(?)", ((Number) constraint.get("oid")).longValue());
                }
            }
            // PostgreTable.initSuperTables, from PostgreTable.getAssociations: the parent tables of items.
            query(c, "SELECT i.*,c.relnamespace FROM pg_catalog.pg_inherits i,pg_catalog.pg_class c WHERE i.inhrelid=? AND c.oid=i.inhparent ORDER BY i.inhseqno", items);
            for (var index : itemIndexes) {
                if (!Boolean.TRUE.equals(index.get("indisprimary"))) {
                    // PostgreIndex.getObjectDefinitionText, from PostgreIndexManager: the text of an index that is not the primary key index.
                    query(c, "SELECT pg_catalog.pg_get_indexdef(?)", ((Number) index.get("indexrelid")).longValue());
                }
            }
            // PostgreTableReal.TriggerCache.prepareLookupStatement: the triggers of items.
            query(c, "SELECT x.oid,x.*,p.pronamespace as func_schema_id,d.description"
                + "\nFROM pg_catalog.pg_trigger x"
                + "\nLEFT OUTER JOIN pg_catalog.pg_proc p ON p.oid=x.tgfoid "
                + "\nLEFT OUTER JOIN pg_catalog.pg_description d ON d.objoid=x.oid AND d.objsubid=0 "
                + "\nWHERE x.tgrelid = ? AND NOT x.tgisinternal", items);
            // PostgreTableReal.RuleCache.prepareObjectsStatement: the rules of items. DBeaver puts the oid in the text.
            query(c, "SELECT r.oid,r.*, pg_get_ruledef(r.oid) AS definition\nFROM pg_rewrite r\nWHERE r.ev_class=" + items + " AND r.rulename <> '_RETURN'::name");
            // PostgreTable.PolicyCache.prepareObjectsStatement: the row level security policies of items.
            query(c, "select * from pg_catalog.pg_policies where schemaname=? and tablename=?", "compat_dbeaver", "items");

            // 4. The data editor reads the first 200 rows of items.
            // QueryTransformerLimit, from PostgreDataSource.createQueryTransformer: DBeaver core adds the LIMIT clause.
            query(c, "SELECT * FROM compat_dbeaver.items\nLIMIT 200");
        }
    }

    // Sends a query with its parameters as DBeaver binds them, drains the result and keeps the rows.
    static List<Map<String, Object>> query(Connection c, String sql, Object... params) throws SQLException {
        try (PreparedStatement p = c.prepareStatement(sql)) {
            for (int i = 0; i < params.length; i++) {
                switch (params[i]) {
                    case Long v -> p.setLong(i + 1, v);
                    case Integer v -> p.setInt(i + 1, v);
                    case String v -> p.setString(i + 1, v);
                    default -> throw new IllegalArgumentException("unknown parameter type: " + params[i]);
                }
            }
            var rows = new ArrayList<Map<String, Object>>();
            try (ResultSet r = p.executeQuery()) {
                var meta = r.getMetaData();
                while (r.next()) {
                    var row = new HashMap<String, Object>();
                    for (int i = 1; i <= meta.getColumnCount(); i++) {
                        row.put(meta.getColumnLabel(i), r.getObject(i));
                    }
                    rows.add(row);
                }
            }
            return rows;
        }
    }

    // Finds the row where the column key has the value name and returns its oid.
    static long oid(List<Map<String, Object>> rows, String key, String name, String oidColumn) {
        for (var row : rows) {
            if (name.equals(row.get(key))) {
                return ((Number) row.get(oidColumn)).longValue();
            }
        }
        throw new IllegalStateException("no row with " + key + " = " + name);
    }
}
