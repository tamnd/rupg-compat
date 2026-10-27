/*
 * The connect program of the connect suite, with libpq.
 *
 * libpq reads PGHOST, PGPORT, PGUSER, PGPASSWORD, PGDATABASE, PGSSLMODE, PGSSLROOTCERT and PGCHANNELBINDING. The program writes the row of the query on one line, with one space between the values.
 */
#include <stdio.h>
#include <stdlib.h>

#include <libpq-fe.h>

int main(void)
{
	PGconn	   *conn = PQconnectdb("");
	PGresult   *res;

	if (PQstatus(conn) != CONNECTION_OK)
	{
		fprintf(stderr, "connect: %s", PQerrorMessage(conn));
		return 1;
	}
	res = PQexec(conn, "SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()");
	if (PQresultStatus(res) != PGRES_TUPLES_OK || PQntuples(res) != 1)
	{
		fprintf(stderr, "query: %s", PQresultErrorMessage(res));
		return 1;
	}
	printf("%s %s %s\n", PQgetvalue(res, 0, 0), PQgetvalue(res, 0, 1), PQgetvalue(res, 0, 2));
	PQclear(res);
	PQfinish(conn);
	return 0;
}
