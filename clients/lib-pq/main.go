// The trace scenario of spec/21 section 21.4.2, with lib/pq. Each client runs the same steps.
package main

import (
	"database/sql"
	"errors"
	"log"

	"github.com/lib/pq"
)

func must(err error) {
	if err != nil {
		log.Fatal(err)
	}
}

// divisionByZero fails unless err is the error 22012.
func divisionByZero(err error) {
	var pqErr *pq.Error
	if !errors.As(err, &pqErr) || pqErr.Code != "22012" {
		log.Fatalf("want 22012, got %v", err)
	}
}

func query(q interface {
	Query(string, ...any) (*sql.Rows, error)
}, text string, args ...any) error {
	rows, err := q.Query(text, args...)
	if err != nil {
		return err
	}
	defer rows.Close()
	cols, err := rows.Columns()
	must(err)
	vals := make([]any, len(cols))
	ptrs := make([]any, len(cols))
	for i := range vals {
		ptrs[i] = &vals[i]
	}
	for rows.Next() {
		must(rows.Scan(ptrs...))
	}
	return rows.Err()
}

func main() {
	// lib/pq reads PGHOST, PGPORT, PGUSER, PGPASSWORD, PGDATABASE and PGSSLMODE.
	db, err := sql.Open("postgres", "")
	must(err)
	defer db.Close()
	// One connection, so that the trace has one session.
	db.SetMaxOpenConns(1)
	// 1. A simple query.
	must(query(db, "SELECT 1"))
	// 2. A query with parameters.
	must(query(db, "SELECT $1::int4 + 1, $2::text", 41, "abc"))
	// 3. The common types.
	must(query(db, "SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true,"+
		" 'x'::text, '\\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date,"+
		" ARRAY[1, 2, 3]::int4[], '{\"a\": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid,"+
		" NULL::text"))
	// 4. A table.
	_, err = db.Exec("CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2))")
	must(err)
	// 5. A transaction that commits.
	tx, err := db.Begin()
	must(err)
	for _, row := range [][]any{{1, "one", "1.50"}, {2, "two", "2.50"}, {3, "three", "3.50"}} {
		_, err = tx.Exec("INSERT INTO compat_items VALUES ($1, $2, $3)", row...)
		must(err)
	}
	must(tx.Commit())
	// 6. A transaction that rolls back.
	tx, err = db.Begin()
	must(err)
	_, err = tx.Exec("INSERT INTO compat_items VALUES ($1, $2, $3)", 4, "four", "4.50")
	must(err)
	must(tx.Rollback())
	// 7. The same statement, run six times.
	for range 6 {
		must(query(db, "SELECT id, name, price FROM compat_items WHERE id >= $1 ORDER BY id", 1))
	}
	// 8. An update.
	_, err = db.Exec("UPDATE compat_items SET price = price * 2 WHERE id = $1", 2)
	must(err)
	// 9. An error, then an error in a transaction.
	divisionByZero(query(db, "SELECT 1 / 0"))
	tx, err = db.Begin()
	must(err)
	divisionByZero(query(tx, "SELECT 1 / 0"))
	must(tx.Rollback())
	// 10. The end.
	_, err = db.Exec("DROP TABLE compat_items")
	must(err)
}
