// The connect program of the connect suite, with lib/pq.
// lib/pq reads PGSSLMODE and PGSSLROOTCERT. It has no channel binding and stops on PGCHANNELBINDING, so the program removes that variable. The program writes the row of the query on one line, with one space between the values.
package main

import (
	"database/sql"
	"fmt"
	"log"
	"os"

	_ "github.com/lib/pq"
)

func main() {
	if err := os.Unsetenv("PGCHANNELBINDING"); err != nil {
		log.Fatal(err)
	}
	db, err := sql.Open("postgres", "")
	if err != nil {
		log.Fatal(err)
	}
	var user, version string
	var ssl bool
	if err := db.QueryRow("SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()").Scan(&user, &ssl, &version); err != nil {
		log.Fatal(err)
	}
	t := "f"
	if ssl {
		t = "t"
	}
	fmt.Println(user, t, version)
	if err := db.Close(); err != nil {
		log.Fatal(err)
	}
}
