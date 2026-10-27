// The connect program of the connect suite, with node-postgres.
// node-postgres does not read PGSSLROOTCERT, so the program gives the certificate. The program writes the row of the query on one line, with one space between the values.
import fs from "node:fs";
import pg from "pg";

const client = new pg.Client({ ssl: { ca: fs.readFileSync(process.env.PGSSLROOTCERT) } });
await client.connect();
const { rows } = await client.query("SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()");
const r = rows[0];
console.log(`${r.system_user} ${r.ssl ? "t" : "f"} ${r.version}`);
await client.end();
