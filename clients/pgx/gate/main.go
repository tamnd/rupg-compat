// The gate program of spec/21 section 21.4.2, with pgx.
//
// pgx has no describe command. So the program runs the catalog queries of clients/gate-queries.sql and writes each row with the Go values that pgx makes. Then, for each table and view of the schema gate, it writes the field descriptions of a query of the relation, and it loads the row type of the relation with LoadType, which reads the catalog with the queries of pgx. A field has the OID of its table, and the program writes the name of the table in place of the OID, because the OIDs of two servers can be different.
package main

import (
	"context"
	"fmt"
	"log"
	"os"
	"strings"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgtype"
)

func must(err error) {
	if err != nil {
		log.Fatal(err)
	}
}

func main() {
	ctx := context.Background()
	// pgx reads PGHOST, PGPORT, PGUSER, PGPASSWORD, PGDATABASE and PGSSLMODE.
	conn, err := pgx.Connect(ctx, "")
	must(err)
	defer conn.Close(ctx)
	text, err := os.ReadFile(os.Args[1])
	must(err)
	for _, line := range strings.Split(string(text), "\n") {
		query := strings.TrimSpace(line)
		if query == "" || strings.HasPrefix(query, "--") {
			continue
		}
		rows, err := conn.Query(ctx, query)
		must(err)
		fmt.Println("query:", query)
		var columns []string
		for _, f := range rows.FieldDescriptions() {
			columns = append(columns, fmt.Sprintf("%s %d", f.Name, f.DataTypeOID))
		}
		fmt.Println("columns:", strings.Join(columns, ", "))
		for rows.Next() {
			values, err := rows.Values()
			must(err)
			fmt.Printf("row: %v\n", values)
		}
		must(rows.Err())
		fmt.Println()
	}
	names := map[uint32]string{}
	var relations []string
	rows, err := conn.Query(ctx, "SELECT oid, relname FROM pg_class WHERE relnamespace = 'gate'::regnamespace AND relkind IN ('r', 'v') ORDER BY relname")
	must(err)
	for rows.Next() {
		var oid uint32
		var name string
		must(rows.Scan(&oid, &name))
		names[oid] = name
		relations = append(relations, name)
	}
	must(rows.Err())
	for _, name := range relations {
		rows, err := conn.Query(ctx, "SELECT * FROM gate."+name+" LIMIT 0")
		must(err)
		fmt.Println("relation:", name)
		for _, f := range rows.FieldDescriptions() {
			table, ok := names[f.TableOID]
			if !ok {
				table = fmt.Sprint(f.TableOID)
			}
			typeName := "unknown"
			if t, ok := conn.TypeMap().TypeForOID(f.DataTypeOID); ok {
				typeName = t.Name
			}
			fmt.Println("column:", f.Name, table, f.TableAttributeNumber, f.DataTypeOID, typeName, f.DataTypeSize, f.TypeModifier, f.Format)
		}
		rows.Close()
		must(rows.Err())
		// LoadType fails for a row type with a field of a type that pgx does not know, such as money. The program writes the error, because it is the same on each server.
		t, err := conn.LoadType(ctx, "gate."+name)
		if err != nil {
			fmt.Println("type:", name, "error:", err)
			fmt.Println()
			continue
		}
		fmt.Println("type:", t.Name)
		if codec, ok := t.Codec.(*pgtype.CompositeCodec); ok {
			for _, field := range codec.Fields {
				fmt.Println("field:", field.Name, field.Type.Name, field.Type.OID)
			}
		}
		fmt.Println()
	}
}
