/*
 * The trace scenario of spec/21 section 21.4.2, with libpq. Each client runs the same steps.
 *
 * libpq reads PGHOST, PGPORT, PGUSER, PGPASSWORD, PGDATABASE and PGSSLMODE.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <libpq-fe.h>

static PGconn *conn;

static void fail(const char *what, PGresult *res)
{
	fprintf(stderr, "%s: %s", what, res ? PQresultErrorMessage(res) : PQerrorMessage(conn));
	exit(1);
}

/* Checks that res has the status want, then frees it. */
static void expect(PGresult *res, ExecStatusType want, const char *what)
{
	if (PQresultStatus(res) != want)
		fail(what, res);
	PQclear(res);
}

/* Runs a statement without parameters. */
static void run(const char *sql, ExecStatusType want)
{
	expect(PQexec(conn, sql), want, sql);
}

/* Checks that sql fails with the error 22012. */
static void division_by_zero(const char *sql)
{
	PGresult *res = PQexec(conn, sql);
	const char *state = PQresultErrorField(res, PG_DIAG_SQLSTATE);

	if (PQresultStatus(res) != PGRES_FATAL_ERROR || state == NULL || strcmp(state, "22012") != 0)
		fail("want 22012", res);
	PQclear(res);
}

/* Inserts one row with text parameters. */
static void insert(const char *id, const char *name, const char *price)
{
	const char *values[3] = {id, name, price};

	expect(PQexecParams(conn, "INSERT INTO compat_items VALUES ($1, $2, $3)", 3, NULL, values, NULL, NULL, 0),
		   PGRES_COMMAND_OK, "insert");
}

int main(void)
{
	conn = PQconnectdb("");
	if (PQstatus(conn) != CONNECTION_OK)
		fail("connect", NULL);

	/* 1. A simple query. */
	run("SELECT 1", PGRES_TUPLES_OK);

	/* 2. A query with parameters. */
	{
		const char *values[2] = {"41", "abc"};

		expect(PQexecParams(conn, "SELECT $1::int4 + 1, $2::text", 2, NULL, values, NULL, NULL, 0),
			   PGRES_TUPLES_OK, "parameters");
	}

	/* 3. The common types. */
	run("SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true,"
		" 'x'::text, '\\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date,"
		" ARRAY[1, 2, 3]::int4[], '{\"a\": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid,"
		" NULL::text",
		PGRES_TUPLES_OK);

	/* 4. A table. */
	run("CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2))", PGRES_COMMAND_OK);

	/* 5. A transaction that commits. */
	run("BEGIN", PGRES_COMMAND_OK);
	insert("1", "one", "1.50");
	insert("2", "two", "2.50");
	insert("3", "three", "3.50");
	run("COMMIT", PGRES_COMMAND_OK);

	/* 6. A transaction that rolls back. */
	run("BEGIN", PGRES_COMMAND_OK);
	insert("4", "four", "4.50");
	run("ROLLBACK", PGRES_COMMAND_OK);

	/* 7. The same statement, run six times. libpq prepares it once, and the rows come in binary format. */
	expect(PQprepare(conn, "items", "SELECT id, name, price FROM compat_items WHERE id >= $1 ORDER BY id", 1, NULL),
		   PGRES_COMMAND_OK, "prepare");
	for (int i = 0; i < 6; i++)
	{
		const char *values[1] = {"1"};

		expect(PQexecPrepared(conn, "items", 1, values, NULL, NULL, 1), PGRES_TUPLES_OK, "execute");
	}

	/* 8. An update. */
	{
		const char *values[1] = {"2"};

		expect(PQexecParams(conn, "UPDATE compat_items SET price = price * 2 WHERE id = $1", 1, NULL, values, NULL,
							NULL, 0),
			   PGRES_COMMAND_OK, "update");
	}

	/* 9. An error, then an error in a transaction. */
	division_by_zero("SELECT 1 / 0");
	run("BEGIN", PGRES_COMMAND_OK);
	division_by_zero("SELECT 1 / 0");
	run("ROLLBACK", PGRES_COMMAND_OK);

	/* 10. The end. */
	run("DROP TABLE compat_items", PGRES_COMMAND_OK);
	PQfinish(conn);
	return 0;
}
