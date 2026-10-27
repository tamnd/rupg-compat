// The connect program of the connect suite, with postgres.js.
// postgres.js does not read PGSSLROOTCERT, so the program gives the certificate. The program writes the row of the query on one line, with one space between the values.
import fs from "node:fs";
import postgres from "postgres";

const sql = postgres({ max: 1, ssl: { ca: fs.readFileSync(process.env.PGSSLROOTCERT) }, onnotice: () => {} });
const [r] = await sql.unsafe("SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()");
console.log(`${r.system_user} ${r.ssl ? "t" : "f"} ${r.version}`);
await sql.end();
