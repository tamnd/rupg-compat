/* The connect program of the connect suite, with psqlODBC through the unixODBC driver manager.
 * The first argument is the path of the driver. psqlODBC sets the SSL mode of libpq from its own SSLMode option, so the program gives verify-full. libpq reads PGSSLROOTCERT and PGCHANNELBINDING. The program writes the row of the query on one line, with one space between the values. */
#include <sql.h>
#include <sqlext.h>
#include <stdio.h>
#include <stdlib.h>

static SQLHENV env;
static SQLHDBC dbc;

/* Prints the diagnostics of a handle and stops, unless the call succeeded. */
static void check(SQLRETURN r, SQLSMALLINT type, SQLHANDLE h, const char *what)
{
    if (SQL_SUCCEEDED(r)) {
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

int main(int argc, char **argv)
{
    if (argc != 2) {
        fprintf(stderr, "usage: connect DRIVER\n");
        return 2;
    }
    char conn[1024];
    snprintf(conn, sizeof conn, "Driver=%s;Server=%s;Port=%s;Database=%s;Uid=%s;Pwd=%s;SSLMode=verify-full", argv[1], getenv("PGHOST"), getenv("PGPORT"), getenv("PGDATABASE"), getenv("PGUSER"), getenv("PGPASSWORD"));
    check(SQLAllocHandle(SQL_HANDLE_ENV, SQL_NULL_HANDLE, &env), SQL_HANDLE_ENV, env, "env");
    check(SQLSetEnvAttr(env, SQL_ATTR_ODBC_VERSION, (SQLPOINTER)SQL_OV_ODBC3, 0), SQL_HANDLE_ENV, env, "version");
    check(SQLAllocHandle(SQL_HANDLE_DBC, env, &dbc), SQL_HANDLE_ENV, env, "dbc");
    check(SQLDriverConnect(dbc, NULL, (SQLCHAR *)conn, SQL_NTS, NULL, 0, NULL, SQL_DRIVER_NOPROMPT), SQL_HANDLE_DBC, dbc, "connect");
    SQLHSTMT st;
    check(SQLAllocHandle(SQL_HANDLE_STMT, dbc, &st), SQL_HANDLE_DBC, dbc, "stmt");
    check(SQLExecDirect(st, (SQLCHAR *)"SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()", SQL_NTS), SQL_HANDLE_STMT, st, "query");
    check(SQLFetch(st), SQL_HANDLE_STMT, st, "fetch");
    SQLCHAR user[256], version[64];
    SQLINTEGER ssl;
    SQLLEN n;
    check(SQLGetData(st, 1, SQL_C_CHAR, user, sizeof user, &n), SQL_HANDLE_STMT, st, "user");
    check(SQLGetData(st, 2, SQL_C_SLONG, &ssl, 0, &n), SQL_HANDLE_STMT, st, "ssl");
    check(SQLGetData(st, 3, SQL_C_CHAR, version, sizeof version, &n), SQL_HANDLE_STMT, st, "version");
    printf("%s %s %s\n", user, ssl ? "t" : "f", version);
    SQLFreeHandle(SQL_HANDLE_STMT, st);
    check(SQLDisconnect(dbc), SQL_HANDLE_DBC, dbc, "disconnect");
    SQLFreeHandle(SQL_HANDLE_DBC, dbc);
    SQLFreeHandle(SQL_HANDLE_ENV, env);
    return 0;
}
