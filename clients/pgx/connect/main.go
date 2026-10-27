// The connect program of the connect suite, with pgx.
// pgx reads PGSSLMODE and PGSSLROOTCERT. The program writes the row of the query on one line, with one space between the values.
package main

import (
	"context"
	"fmt"
	"log"

	"github.com/jackc/pgx/v5"
)

func main() {
	ctx := context.Background()
	conn, err := pgx.Connect(ctx, "")
	if err != nil {
		log.Fatal(err)
	}
	var user, version string
	var ssl bool
	if err := conn.QueryRow(ctx, "SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()").Scan(&user, &ssl, &version); err != nil {
		log.Fatal(err)
	}
	t := "f"
	if ssl {
		t = "t"
	}
	fmt.Println(user, t, version)
	if err := conn.Close(ctx); err != nil {
		log.Fatal(err)
	}
}
