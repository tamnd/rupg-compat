// The trace scenario of spec/21 section 21.4.2, with pgx. Each client runs the same steps.
package main

import (
	"context"
	"errors"
	"log"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
)

func must(err error) {
	if err != nil {
		log.Fatal(err)
	}
}

// divisionByZero fails unless err is the error 22012.
func divisionByZero(err error) {
	var pgErr *pgconn.PgError
	if !errors.As(err, &pgErr) || pgErr.Code != "22012" {
		log.Fatalf("want 22012, got %v", err)
	}
}

func query(ctx context.Context, conn *pgx.Conn, sql string, args ...any) {
	rows, err := conn.Query(ctx, sql, args...)
	must(err)
	for rows.Next() {
		_, err := rows.Values()
		must(err)
	}
	must(rows.Err())
}

func main() {
	ctx := context.Background()
	// pgx reads PGHOST, PGPORT, PGUSER, PGPASSWORD, PGDATABASE and PGSSLMODE.
	conn, err := pgx.Connect(ctx, "")
	must(err)
	defer conn.Close(ctx)
	// 1. A simple query.
	query(ctx, conn, "SELECT 1")
	// 2. A query with parameters.
	query(ctx, conn, "SELECT $1::int4 + 1, $2::text", 41, "abc")
	// 3. The common types.
	query(ctx, conn, "SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true,"+
		" 'x'::text, '\\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date,"+
		" ARRAY[1, 2, 3]::int4[], '{\"a\": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid,"+
		" NULL::text")
	// 4. A table.
	_, err = conn.Exec(ctx, "CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2))")
	must(err)
	// 5. A transaction that commits.
	tx, err := conn.Begin(ctx)
	must(err)
	for _, row := range [][]any{{1, "one", "1.50"}, {2, "two", "2.50"}, {3, "three", "3.50"}} {
		_, err = tx.Exec(ctx, "INSERT INTO compat_items VALUES ($1, $2, $3)", row...)
		must(err)
	}
	must(tx.Commit(ctx))
	// 6. A transaction that rolls back.
	tx, err = conn.Begin(ctx)
	must(err)
	_, err = tx.Exec(ctx, "INSERT INTO compat_items VALUES ($1, $2, $3)", 4, "four", "4.50")
	must(err)
	must(tx.Rollback(ctx))
	// 7. The same statement, run six times.
	for range 6 {
		query(ctx, conn, "SELECT id, name, price FROM compat_items WHERE id >= $1 ORDER BY id", 1)
	}
	// 8. An update.
	_, err = conn.Exec(ctx, "UPDATE compat_items SET price = price * 2 WHERE id = $1", 2)
	must(err)
	// 9. An error, then an error in a transaction.
	_, err = conn.Exec(ctx, "SELECT 1 / 0")
	divisionByZero(err)
	tx, err = conn.Begin(ctx)
	must(err)
	_, err = tx.Exec(ctx, "SELECT 1 / 0")
	divisionByZero(err)
	must(tx.Rollback(ctx))
	// 10. The end.
	_, err = conn.Exec(ctx, "DROP TABLE compat_items")
	must(err)
}
