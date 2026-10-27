// The trace scenario for Beekeeper Studio 6.1.6. Each query is the SQL of apps/studio/src/lib/db/clients/postgresql.ts, in the order in which the app sends it.
// The app sends the queries through a pg.Pool. This scenario sends them one at a time on one client, so the order on the wire is fixed.
// The SQL takes the branches for a server that reports version 19.0. The app reads that version as the number 190000.
// node-postgres reads PGHOST, PGPORT, PGUSER, PGPASSWORD and PGDATABASE.
import pg from "pg";

// The app sets these type parsers when it loads its PostgreSQL client.
pg.types.setTypeParser(pg.types.builtins.DATE, "text", (val) => val);
pg.types.setTypeParser(pg.types.builtins.TIMESTAMP, "text", (val) => val);
pg.types.setTypeParser(pg.types.builtins.TIMESTAMPTZ, "text", (val) => val);
pg.types.setTypeParser(pg.types.builtins.INTERVAL, "text", (val) => val);

const client = new pg.Client();
await client.connect();

// runQuery gives node-postgres the text and the parameters. Without parameters, node-postgres sends a simple query.
function run(text, params) {
  return client.query({ text, values: params });
}

const schema = "compat_beekeeper";
// The sidebar sends its lists with no filter. The empty filter keeps the indent of its line in the SQL.
const noFilter = "";

// listTableColumns and listEnumValues. The store action updateTableColumns sends them together.
async function updateTableColumns(table) {
  await run(`
      SELECT
        table_schema,
        table_name,
        column_name,
        is_nullable,
        is_generated,
          ordinal_position,
        column_default,
        CASE
          WHEN character_maximum_length is not null  and udt_name != 'text'
            THEN udt_name || '(' || character_maximum_length::varchar(255) || ')'
          WHEN numeric_precision is not null and numeric_scale is not null
            THEN udt_name || '(' || numeric_precision::varchar(255) || ',' || numeric_scale::varchar(255) || ')'
          WHEN numeric_precision is not null and numeric_scale is null
            THEN udt_name || '(' || numeric_precision::varchar(255) || ')'
          WHEN datetime_precision is not null AND udt_name != 'date' THEN
            udt_name || '(' || datetime_precision::varchar(255) || ')'
          ELSE udt_name
      END as data_type,
        udt_schema,
        CASE
          WHEN data_type = 'ARRAY' THEN 'YES'
          ELSE 'NO'
      END as is_array,
        pg_catalog.col_description(format('%I.%I', table_schema, table_name)::regclass::oid, ordinal_position) as column_comment
      FROM information_schema.columns
      WHERE table_schema = $1 AND table_name = $2
      ORDER BY table_schema, table_name, ordinal_position
    `, [schema, table]);
  await run(`
        SELECT n.nspname AS schema, t.typname AS typename, e.enumlabel AS label
        FROM pg_enum e
        JOIN pg_type t ON t.oid = e.enumtypid
        JOIN pg_namespace n ON n.oid = t.typnamespace
        WHERE t.oid IN (
             SELECT COALESCE(NULLIF(et.typelem, 0), et.oid)
             FROM pg_attribute a
             JOIN pg_class c ON c.oid = a.attrelid
             JOIN pg_namespace cn ON cn.oid = c.relnamespace
             JOIN pg_type et ON et.oid = a.atttypid
             WHERE c.relname = $1 AND cn.nspname = $2
               AND a.attnum > 0 AND NOT a.attisdropped
           )
        ORDER BY n.nspname, t.typname, e.enumsortorder
      `, [table, schema]);
}

// getOutgoingKeys, through getTableKeys of the base class.
async function getTableKeys(table) {
  await run(`
      SELECT
        c.conname AS constraint_name,
        a.attname AS column_name,
        n.nspname AS from_schema,
        t.relname AS from_table,
        af.attname AS to_column,
        tf.relname AS to_table,
        nf.nspname AS to_schema,
        CASE c.confupdtype::text
          WHEN 'a' THEN 'NO ACTION'
          WHEN 'r' THEN 'RESTRICT'
          WHEN 'c' THEN 'CASCADE'
          WHEN 'n' THEN 'SET NULL'
          WHEN 'd' THEN 'SET DEFAULT'
          ELSE c.confupdtype::text
        END AS update_rule,
        CASE c.confdeltype::text
          WHEN 'a' THEN 'NO ACTION'
          WHEN 'r' THEN 'RESTRICT'
          WHEN 'c' THEN 'CASCADE'
          WHEN 'n' THEN 'SET NULL'
          WHEN 'd' THEN 'SET DEFAULT'
          ELSE c.confdeltype::text
        END AS delete_rule,
        pos AS ordinal_position
      FROM
        pg_constraint c
        JOIN pg_class t ON c.conrelid = t.oid
        JOIN pg_namespace n ON t.relnamespace = n.oid
        JOIN generate_subscripts(c.conkey, 1) pos ON true
        JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = c.conkey[pos]
        JOIN pg_class tf ON c.confrelid = tf.oid
        JOIN pg_namespace nf ON tf.relnamespace = nf.oid
        JOIN pg_attribute af ON af.attrelid = tf.oid AND af.attnum = c.confkey[pos]
      WHERE
        c.contype = 'f'
        AND n.nspname = $1
        AND t.relname = $2
      ORDER BY
        c.conname,
        pos;
    `, [schema, table]);
}

// getPrimaryKeys. The app puts the table name in the SQL as a literal.
async function getPrimaryKeys(table) {
  await run(`
      SELECT
        a.attname as column_name,
        format_type(a.atttypid, a.atttypmod) AS data_type,
        a.attnum as position
      FROM   pg_index i
        JOIN   pg_attribute a ON a.attrelid = i.indrelid
        AND a.attnum = ANY(i.indkey)
      WHERE  i.indrelid = '"${schema}"."${table}"'::regclass
        AND    i.indisprimary
      ORDER BY a.attnum
    `);
}

// 1. The connection. connect gets a client from the pool and releases it, then it sends these three queries.
// getSchema.
await run("SELECT CURRENT_SCHEMA() AS schema");
// getVersion.
await run("select version()");
// getTypes.
await run(`
      SELECT      n.nspname as schema, t.typname as typename, t.oid::integer as typeid
      FROM        pg_type t
        LEFT JOIN   pg_catalog.pg_namespace n ON n.oid = t.typnamespace
      WHERE       (t.typrelid = 0 OR (SELECT c.relkind = 'c' FROM pg_catalog.pg_class c WHERE c.oid = t.typrelid))
        AND     n.nspname NOT IN ('pg_catalog', 'information_schema')
     AND     NOT EXISTS(SELECT 1 FROM pg_catalog.pg_type el WHERE el.oid = t.typelem AND el.typarray = t.oid);`);

// 2. The sidebar. The store sends updateDatabaseList, updateTables and updateRoutines with no filter.
// listDatabases.
await run(`
      SELECT datname
      FROM pg_database
      WHERE datistemplate = $1
        ${noFilter}
      ORDER BY datname
    `, [false]);
// listSchemas. The app 6.1.6 sends it for plugins. The sidebar gets the schemas from the list of tables.
await run(`
      SELECT schema_name
      FROM information_schema.schemata
             ${noFilter}
      ORDER BY schema_name
    `);
// listTables, with the pg_class branch for servers from version 10.
await run(`
      SELECT
        t.table_schema as schema,
        t.table_name as name,
    ` + `
          pc.relkind as tabletype,
          parent_pc.relkind as parenttype
        FROM information_schema.tables AS t
        JOIN pg_class AS pc
          ON t.table_name = pc.relname AND quote_ident(t.table_schema) = pc.relnamespace::regnamespace::text
        LEFT OUTER JOIN pg_inherits AS i
          ON pc.oid = i.inhrelid
        LEFT OUTER JOIN pg_class AS parent_pc
          ON parent_pc.oid = i.inhparent
        WHERE t.table_type NOT LIKE '%VIEW%'
      ` + `
      ${noFilter}
      ORDER BY t.table_schema, t.table_name
    `);
// listViews.
await run(`
      SELECT
        table_schema as schema,
        table_name as name
      FROM information_schema.views
        ${noFilter}
      ORDER BY table_schema, table_name
    `);
// listMaterializedViews.
await run(`
      SELECT
        schemaname as schema,
        matviewname as name
      FROM pg_matviews
        ${noFilter}
      order by schemaname, matviewname;
    `);
// listRoutines, the routines.
const routines = await run(`
      SELECT
        r.specific_name as id,
        r.routine_schema as routine_schema,
        r.routine_name as name,
        r.routine_type as routine_type,
        r.data_type as data_type,
        p.oid::text as oid
      FROM INFORMATION_SCHEMA.ROUTINES r
      LEFT JOIN pg_catalog.pg_proc p
        ON r.specific_name::text = p.proname || '_' || p.oid::text
      where r.routine_schema not in ('sys', 'information_schema',
                                     'pg_catalog', 'performance_schema')
        ${noFilter}
      ORDER BY routine_schema, routine_name
    `);
// listRoutines, the parameters of the routines.
await run(`
      select
        r.routine_schema as routine_schema,
        r.specific_name as specific_name,
        p.parameter_name as parameter_name,
        p.character_maximum_length as char_length,
          p.data_type as data_type
      from information_schema.routines r
        left join information_schema.parameters p
      on p.specific_schema = r.routine_schema
        and p.specific_name = r.specific_name
      where r.routine_schema not in ('sys', 'information_schema',
        'pg_catalog', 'performance_schema')
        ${noFilter}

        AND p.parameter_mode = 'IN'
      order by r.routine_schema,
        r.specific_name,
        p.ordinal_position;

    `);
// The user opens compat_orders in the sidebar. The store action updateTableColumns gets its columns.
await updateTableColumns("compat_orders");

// 3. The structure view of compat_orders. TabTableProperties.vue gets the columns, the primary keys and the table properties.
await updateTableColumns("compat_orders");
await getPrimaryKeys("compat_orders");
// getTableProperties, the sizes and the comment.
await run(`SELECT pg_indexes_size('"${schema}"."compat_orders"') as index_size,pg_relation_size('"${schema}"."compat_orders"') as table_size,obj_description('"${schema}"."compat_orders"'::regclass) as description`);
// listTableIndexes, with indnullsnotdistinct for servers from version 15.
await run(`
      SELECT (SELECT relname FROM pg_class c WHERE c.oid = i.indexrelid) as indexname,
             k.i AS index_order,
             i.indexrelid as id,
             i.indisunique,
             i.indisprimary,
             i.indnullsnotdistinct,
        coalesce(a.attname, pg_get_indexdef(i.indexrelid, k.i, false)) AS index_column,
        i.indoption[k.i - 1] = 0 AS ascending
      FROM pg_index i
        CROSS JOIN LATERAL (SELECT unnest(i.indkey), generate_subscripts(i.indkey, 1) + 1) AS k(attnum, i)
        LEFT JOIN pg_attribute AS a
      ON i.indrelid = a.attrelid AND k.attnum = a.attnum
        JOIN pg_class t on t.oid = i.indrelid
        JOIN pg_namespace c on c.oid = t.relnamespace
      WHERE
        c.nspname = $1 AND
        t.relname = $2

    `, [schema, "compat_orders"]);
await getTableKeys("compat_orders");
// listTableTriggers, with action_timing for servers after version 9.0.
await run(`
      SELECT
        trigger_name,
        action_timing as timing,
        event_manipulation as manipulation,
        action_statement as action,
        action_condition as condition
      FROM information_schema.triggers
      WHERE event_object_schema = $1
        AND event_object_table = $2
    `, [schema, "compat_orders"]);
// listTablePartitions. knex puts the two values in the SQL as literals.
await run(`
      SELECT
        ps.schemaname AS schema,
          ps.relname AS name,
          pg_get_expr(pt.relpartbound, pt.oid, true) AS expression
      FROM pg_class base_tb
        JOIN pg_inherits i              ON i.inhparent = base_tb.oid
        JOIN pg_class pt                ON pt.oid = i.inhrelid
        JOIN pg_stat_all_tables ps      ON ps.relid = i.inhrelid
        JOIN pg_namespace nmsp_parent   ON nmsp_parent.oid = base_tb.relnamespace
      WHERE nmsp_parent.nspname = '${schema}' AND base_tb.relname = 'compat_orders' AND base_tb.relkind = 'p';
    `);
// getTableOwner.
await run("select tableowner from pg_catalog.pg_tables where tablename = $1 and schemaname = $2", ["compat_orders", schema]);
// getTableCreateScript, with postgres10CreateScript of postgresql/scripts.ts. The sidebar menu shows this DDL.
await run(`
  SELECT
  'CREATE TABLE ' || quote_ident(tabdef.schema_name) || '.' || quote_ident(tabdef.table_name) || E' (\n' ||
  array_to_string(
    array_agg(
      '  ' || quote_ident(tabdef.column_name) || ' ' ||
      case when tabdef.def_val like 'nextval(%_seq%' then
        case when tabdef.type = 'integer' then 'serial'
            when tabdef.type = 'smallint' then 'smallserial'
            when tabdef.type = 'bigint' then 'bigserial'
            else tabdef.type end
      else
        tabdef.type
      end || ' ' ||
      tabdef.not_null ||
      CASE WHEN tabdef.def_val IS NOT NULL
                AND NOT (tabdef.def_val like 'nextval(%_seq%'
                        AND (tabdef.type = 'integer' OR tabdef.type = 'smallint' OR tabdef.type = 'bigint'))
          THEN ' DEFAULT ' || tabdef.def_val
      ELSE '' END ||
      CASE WHEN tabdef.identity IS NOT NULL THEN ' ' || tabdef.identity ELSE '' END
      ORDER BY tabdef.column_idx ASC
    )
    , E',\n'
  ) || E'\n);\n' ||
  CASE WHEN tc.constraint_name IS NULL THEN ''
      ELSE E'\nALTER TABLE ' || quote_ident(tabdef.schema_name) || '.' || quote_ident(tabdef.table_name) ||
      ' ADD CONSTRAINT ' || quote_ident(tc.constraint_name)  ||
      ' PRIMARY KEY ' || '(' || substring(constr.column_name from 0 for char_length(constr.column_name)-1) || ')'
  END AS createtable
  FROM
  ( SELECT
    c.relname AS table_name,
    a.attname AS column_name,
    a.attnum AS column_idx,
    pg_catalog.format_type(a.atttypid, a.atttypmod) AS type,
    CASE
      WHEN a.attnotnull OR a.attidentity != '' THEN 'NOT NULL'
    ELSE 'NULL'
    END AS not_null,
    CASE WHEN a.atthasdef THEN pg_catalog.pg_get_expr(ad.adbin, ad.adrelid) ELSE null END AS def_val,
    CASE WHEN a.attidentity = 'a' THEN 'GENERATED ALWAYS AS IDENTITY' when a.attidentity = 'd' THEN 'GENERATED BY DEFAULT AS IDENTITY' ELSE null END AS identity,
    n.nspname as schema_name
  FROM pg_class c
  JOIN pg_namespace n ON (n.oid = c.relnamespace)
  JOIN pg_attribute a ON (a.attnum > 0 AND a.attrelid = c.oid)
  JOIN pg_type t ON (a.atttypid = t.oid)
  LEFT JOIN pg_attrdef ad ON (a.attrelid = ad.adrelid AND a.attnum = ad.adnum)
  WHERE c.relname = $1
  AND n.nspname = $2
  ORDER BY a.attnum DESC
  ) AS tabdef
  LEFT JOIN information_schema.table_constraints tc
  ON  tc.table_name       = tabdef.table_name
  AND tc.table_schema     = tabdef.schema_name
  AND tc.constraint_Type  = 'PRIMARY KEY'
  LEFT JOIN LATERAL (
  SELECT column_name || ', ' AS column_name
  FROM   information_schema.key_column_usage kcu
  WHERE  kcu.constraint_name = tc.constraint_name
  AND kcu.table_name = tabdef.table_name
  AND kcu.table_schema = tabdef.schema_name
  ORDER BY ordinal_position
  ) AS constr ON true
  GROUP BY tabdef.schema_name, tabdef.table_name, tc.constraint_name, constr.column_name;

`, ["compat_orders", schema]);
// getViewCreateScript. The sidebar menu shows this DDL for the view.
await run("SELECT pg_get_viewdef($1::regclass, true)", [`"${schema}"."compat_item_totals"`]);
// getRoutineCreateScript. The app gives the oid that listRoutines returned.
const routine = routines.rows.find((r) => r.routine_schema === schema && r.name === "compat_item_qty");
await run(`
      SELECT pg_get_functiondef(p.oid)
      FROM pg_proc p
             LEFT JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace
      WHERE proname = $1
        AND n.nspname = $2
        AND p.oid = $3::oid
    `, ["compat_item_qty", schema, routine.oid]);

// 4. The table view of compat_items. TableTable.vue gets the columns and the keys, then the first page.
await updateTableColumns("compat_items");
await getTableKeys("compat_items");
await getPrimaryKeys("compat_items");
// selectTop, through buildSelectTopQueries. The page size is 100 and the app asks for one more row. The initial sort is the first column, and the filter is empty.
const baseSQL = `
      FROM "${schema}"."compat_items"
      ${noFilter}
    `;
await run(`
      SELECT * ${baseSQL}
      ORDER BY "id" ASC
      LIMIT 101
      OFFSET 0
      `, []);
// TableTable.vue compares the fields with the columns after each page, and the comparison always finds a difference.
await updateTableColumns("compat_items");
await client.end();
