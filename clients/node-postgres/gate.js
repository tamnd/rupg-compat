// The gate program of spec/21 section 21.4.2, with node-postgres.
// node-postgres has no describe command. So the program runs the catalog queries of clients/gate-queries.sql and writes each row with the JavaScript values that node-postgres makes. Then, for each table and view of the schema gate, it writes the fields of the result of a query of the relation. A field has the OID of its table, and the program writes the name of the table in place of the OID, because the OIDs of two servers can be different.
import { readFileSync } from "node:fs";
import { inspect } from "node:util";
import pg from "pg";

const client = new pg.Client();
await client.connect();
const queries = readFileSync(process.argv[2], "utf8")
  .split("\n")
  .map((line) => line.trim())
  .filter((line) => line && !line.startsWith("--"));
for (const query of queries) {
  const result = await client.query({ text: query, rowMode: "array" });
  console.log("query:", query);
  console.log("columns:", inspect(result.fields.map((f) => [f.name, f.dataTypeID])));
  for (const row of result.rows) console.log("row:", inspect(row, { breakLength: Infinity }));
  console.log();
}
const names = new Map();
const relations = await client.query(
  "SELECT oid::int8::text AS oid, relname FROM pg_class WHERE relnamespace = 'gate'::regnamespace AND relkind IN ('r', 'v') ORDER BY relname",
);
for (const r of relations.rows) names.set(Number(r.oid), r.relname);
for (const r of relations.rows) {
  const result = await client.query(`SELECT * FROM gate.${r.relname} LIMIT 0`);
  console.log("relation:", r.relname);
  for (const f of result.fields) {
    const table = names.get(f.tableID) ?? f.tableID;
    console.log("column:", f.name, table, f.columnID, f.dataTypeID, f.dataTypeSize, f.dataTypeModifier, f.format);
  }
  console.log();
}
await client.end();
