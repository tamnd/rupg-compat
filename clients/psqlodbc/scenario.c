/* The trace scenario of spec/21 section 21.4.2, with psqlODBC through the unixODBC driver manager. Each driver runs the same 10 steps.
 * The first argument is the path of the driver. ODBC marks a parameter with ?, and psqlODBC prepares the statements on the server. */
#include <sql.h>
#include <sqlext.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static SQLHENV env;
static SQLHDBC dbc;

/* Prints the diagnostics of a handle and stops, unless the call succeeded. */
static void check(SQLRETURN r, SQLSMALLINT type, SQLHANDLE h, const char *what)
{
    if (SQL_SUCCEEDED(r) || r == SQL_NO_DATA) {
        return;
    }
    SQLCHAR state[6], text[512];
    SQLINTEGER native;
    SQLSMALLINT len;
    for (SQLSMALLINT i = 1; SQLGetDiagRec(type, h, i, state, &native, text, sizeof text, &len) == SQL_SUCCESS; i++) {
        fprintf(stderr, "%s: %s %s\n", what, state, text);
    }
    exit(1);
}

static SQLHSTMT statement(void)
{
    SQLHSTMT s;
    check(SQLAllocHandle(SQL_HANDLE_STMT, dbc, &s), SQL_HANDLE_DBC, dbc, "statement");
    return s;
}

/* Reads every row and every column of the result, as text. */
static void drain(SQLHSTMT s)
{
    SQLSMALLINT columns;
    check(SQLNumResultCols(s, &columns), SQL_HANDLE_STMT, s, "columns");
    SQLRETURN r;
    while ((r = SQLFetch(s)) == SQL_SUCCESS || r == SQL_SUCCESS_WITH_INFO) {
        for (SQLUSMALLINT i = 1; i <= (SQLUSMALLINT)columns; i++) {
            char value[256];
            SQLLEN ind;
            check(SQLGetData(s, i, SQL_C_CHAR, value, sizeof value, &ind), SQL_HANDLE_STMT, s, "value");
        }
    }
    check(r, SQL_HANDLE_STMT, s, "fetch");
    check(SQLFreeStmt(s, SQL_CLOSE), SQL_HANDLE_STMT, s, "close");
}

static void exec(const char *sql)
{
    SQLHSTMT s = statement();
    check(SQLExecDirect(s, (SQLCHAR *)sql, SQL_NTS), SQL_HANDLE_STMT, s, sql);
    drain(s);
    SQLFreeHandle(SQL_HANDLE_STMT, s);
}

static void autocommit(int on)
{
    check(SQLSetConnectAttr(dbc, SQL_ATTR_AUTOCOMMIT, (SQLPOINTER)(on ? SQL_AUTOCOMMIT_ON : SQL_AUTOCOMMIT_OFF), 0), SQL_HANDLE_DBC, dbc, "autocommit");
}

static void end(SQLSMALLINT how)
{
    check(SQLEndTran(SQL_HANDLE_DBC, dbc, how), SQL_HANDLE_DBC, dbc, "end transaction");
}

static void insert(SQLHSTMT s, SQLINTEGER id, const char *name, const char *price)
{
    SQLLEN nts = SQL_NTS;
    check(SQLBindParameter(s, 1, SQL_PARAM_INPUT, SQL_C_SLONG, SQL_INTEGER, 0, 0, &id, 0, NULL), SQL_HANDLE_STMT, s, "bind id");
    check(SQLBindParameter(s, 2, SQL_PARAM_INPUT, SQL_C_CHAR, SQL_VARCHAR, 100, 0, (SQLPOINTER)name, 0, &nts), SQL_HANDLE_STMT, s, "bind name");
    check(SQLBindParameter(s, 3, SQL_PARAM_INPUT, SQL_C_CHAR, SQL_NUMERIC, 10, 2, (SQLPOINTER)price, 0, &nts), SQL_HANDLE_STMT, s, "bind price");
    check(SQLExecute(s), SQL_HANDLE_STMT, s, "insert");
}

/* Runs SELECT 1 / 0 and wants SQLSTATE 22012. */
static void want22012(void)
{
    SQLHSTMT s = statement();
    if (SQL_SUCCEEDED(SQLExecDirect(s, (SQLCHAR *)"SELECT 1 / 0", SQL_NTS))) {
        fprintf(stderr, "want division by zero\n");
        exit(1);
    }
    SQLCHAR state[6], text[512];
    SQLINTEGER native;
    SQLSMALLINT len;
    check(SQLGetDiagRec(SQL_HANDLE_STMT, s, 1, state, &native, text, sizeof text, &len), SQL_HANDLE_STMT, s, "diagnostics");
    if (strcmp((char *)state, "22012") != 0) {
        fprintf(stderr, "want 22012, got %s %s\n", state, text);
        exit(1);
    }
    SQLFreeHandle(SQL_HANDLE_STMT, s);
}

int main(int argc, char **argv)
{
    if (argc != 2) {
        fprintf(stderr, "usage: scenario DRIVER\n");
        return 2;
    }
    char conn[1024];
    snprintf(conn, sizeof conn, "Driver=%s;Server=%s;Port=%s;Database=%s;Uid=%s;Pwd=%s;SSLMode=disable", argv[1], getenv("PGHOST"), getenv("PGPORT"), getenv("PGDATABASE"), getenv("PGUSER"), getenv("PGPASSWORD"));
    check(SQLAllocHandle(SQL_HANDLE_ENV, SQL_NULL_HANDLE, &env), SQL_HANDLE_ENV, env, "environment");
    check(SQLSetEnvAttr(env, SQL_ATTR_ODBC_VERSION, (SQLPOINTER)SQL_OV_ODBC3, 0), SQL_HANDLE_ENV, env, "version");
    check(SQLAllocHandle(SQL_HANDLE_DBC, env, &dbc), SQL_HANDLE_ENV, env, "connection");
    check(SQLDriverConnect(dbc, NULL, (SQLCHAR *)conn, SQL_NTS, NULL, 0, NULL, SQL_DRIVER_NOPROMPT), SQL_HANDLE_DBC, dbc, "connect");

    /* 1. A simple query. */
    exec("SELECT 1");
    /* 2. A query with parameters. */
    SQLHSTMT s = statement();
    SQLINTEGER n = 41;
    SQLLEN nts = SQL_NTS;
    check(SQLPrepare(s, (SQLCHAR *)"SELECT ?::int4 + 1, ?::text", SQL_NTS), SQL_HANDLE_STMT, s, "prepare");
    check(SQLBindParameter(s, 1, SQL_PARAM_INPUT, SQL_C_SLONG, SQL_INTEGER, 0, 0, &n, 0, NULL), SQL_HANDLE_STMT, s, "bind");
    check(SQLBindParameter(s, 2, SQL_PARAM_INPUT, SQL_C_CHAR, SQL_VARCHAR, 3, 0, (SQLPOINTER) "abc", 0, &nts), SQL_HANDLE_STMT, s, "bind");
    check(SQLExecute(s), SQL_HANDLE_STMT, s, "execute");
    drain(s);
    SQLFreeHandle(SQL_HANDLE_STMT, s);
    /* 3. The common types. */
    exec("SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true, 'x'::text, '\\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date, ARRAY[1, 2, 3]::int4[], '{\"a\": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid, NULL::text");
    /* 4. A table. */
    exec("CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2))");
    /* 5. A transaction that commits. */
    autocommit(0);
    s = statement();
    check(SQLPrepare(s, (SQLCHAR *)"INSERT INTO compat_items VALUES (?, ?, ?)", SQL_NTS), SQL_HANDLE_STMT, s, "prepare");
    insert(s, 1, "one", "1.50");
    insert(s, 2, "two", "2.50");
    insert(s, 3, "three", "3.50");
    end(SQL_COMMIT);
    /* 6. A transaction that rolls back. */
    insert(s, 4, "four", "4.50");
    end(SQL_ROLLBACK);
    SQLFreeHandle(SQL_HANDLE_STMT, s);
    autocommit(1);
    /* 7. A prepared statement, run six times. */
    s = statement();
    SQLINTEGER low = 1;
    check(SQLPrepare(s, (SQLCHAR *)"SELECT id, name, price FROM compat_items WHERE id >= ? ORDER BY id", SQL_NTS), SQL_HANDLE_STMT, s, "prepare");
    check(SQLBindParameter(s, 1, SQL_PARAM_INPUT, SQL_C_SLONG, SQL_INTEGER, 0, 0, &low, 0, NULL), SQL_HANDLE_STMT, s, "bind");
    for (int i = 0; i < 6; i++) {
        check(SQLExecute(s), SQL_HANDLE_STMT, s, "execute");
        drain(s);
    }
    SQLFreeHandle(SQL_HANDLE_STMT, s);
    /* 8. An update. */
    s = statement();
    SQLINTEGER id = 2;
    check(SQLPrepare(s, (SQLCHAR *)"UPDATE compat_items SET price = price * 2 WHERE id = ?", SQL_NTS), SQL_HANDLE_STMT, s, "prepare");
    check(SQLBindParameter(s, 1, SQL_PARAM_INPUT, SQL_C_SLONG, SQL_INTEGER, 0, 0, &id, 0, NULL), SQL_HANDLE_STMT, s, "bind");
    check(SQLExecute(s), SQL_HANDLE_STMT, s, "update");
    SQLFreeHandle(SQL_HANDLE_STMT, s);
    /* 9. An error, then an error in a transaction. */
    want22012();
    autocommit(0);
    want22012();
    end(SQL_ROLLBACK);
    autocommit(1);
    /* 10. The end. */
    exec("DROP TABLE compat_items");
    check(SQLDisconnect(dbc), SQL_HANDLE_DBC, dbc, "disconnect");
    SQLFreeHandle(SQL_HANDLE_DBC, dbc);
    SQLFreeHandle(SQL_HANDLE_ENV, env);
    return 0;
}
