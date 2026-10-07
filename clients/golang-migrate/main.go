// The trace scenario of spec/21 section 21.4.2, with golang-migrate and its pgx/v5 driver.
//
// A migration tool does not send the same statements as a driver. The scenario runs the migrations of migrations/ up, reads the version, then runs them down.
package main

import (
	"embed"
	"errors"
	"fmt"
	"log"
	"net/url"
	"os"

	"github.com/golang-migrate/migrate/v4"
	_ "github.com/golang-migrate/migrate/v4/database/pgx/v5"
	"github.com/golang-migrate/migrate/v4/source/iofs"
)

//go:embed migrations/*.sql
var migrations embed.FS

func must(err error) {
	if err != nil && !errors.Is(err, migrate.ErrNoChange) {
		log.Fatal(err)
	}
}

func main() {
	// golang-migrate takes a URL, so the scenario builds it from the libpq environment.
	u := url.URL{
		Scheme:   "pgx5",
		User:     url.UserPassword(os.Getenv("PGUSER"), os.Getenv("PGPASSWORD")),
		Host:     fmt.Sprintf("%s:%s", os.Getenv("PGHOST"), os.Getenv("PGPORT")),
		Path:     "/" + os.Getenv("PGDATABASE"),
		RawQuery: "sslmode=" + os.Getenv("PGSSLMODE"),
	}
	src, err := iofs.New(migrations, "migrations")
	must(err)
	m, err := migrate.NewWithSourceInstance("iofs", src, u.String())
	must(err)
	// 1. All migrations up.
	must(m.Up())
	// 2. The version.
	v, dirty, err := m.Version()
	must(err)
	if v != 2 || dirty {
		log.Fatalf("want version 2, clean, got %d, %v", v, dirty)
	}
	// 3. All migrations down.
	must(m.Down())
	srcErr, dbErr := m.Close()
	must(srcErr)
	must(dbErr)
}
