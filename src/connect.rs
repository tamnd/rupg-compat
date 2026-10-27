//! L1: psql, libpq and the drivers of the connect matrix connect with SCRAM over TLS (spec/05 section 5.13).
//!
//! The runner of the gate runs `clients/<name>/run.sh connect` for each client, on the oracle and on the server under test. The program connects to the database `postgres` with TLS, checks the certificate of the oracle in `PGSSLROOTCERT` and the name `127.0.0.1`, and gives the SCRAM password. Then it runs the query `QUERY` and writes the three values of its row on one line, with one space between them, and disconnects. The values show the method and the user of the authentication, and that the server sees TLS and its version. A client passes when the line on the server under test is equal to the line on the oracle.
//!
//! The recording proxy does not speak TLS, so the traces of L1 are recorded without TLS. This suite checks the part that the traces cannot show: the TLS handshake, SCRAM with TLS and the end of the session.

use crate::gate::Suite;

/// The connect suite.
pub(crate) const SUITE: Suite =
    Suite { name: "connect", database: "postgres", schema: None, tls: true };
/// The query of each connect program.
#[cfg(test)]
const QUERY: &str =
    "SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()";
/// psql, libpq and the drivers of the connect matrix.
pub(crate) const CLIENTS: [&str; 7] =
    ["psql", "libpq", "pgjdbc", "psycopg", "node-postgres", "pgx", "asyncpg"];

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::fails;

    #[test]
    fn each_client_has_a_connect_program_with_the_query() {
        for name in CLIENTS {
            let dir = Path::new("clients").join(name);
            let script = fs::read_to_string(dir.join("run.sh")).unwrap();
            assert!(script.contains("connect"), "clients/{name}/run.sh has no connect program");
            // The program is a file `connect.<ext>` or `Connect.<ext>`, or a directory `connect` with the source.
            let mut files: Vec<_> =
                fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
            if let Ok(sub) = fs::read_dir(dir.join("connect")) {
                files.extend(sub.map(|e| e.unwrap().path()));
            }
            let found = files.iter().any(|path| {
                let program = path.file_stem().is_some_and(|s| s.eq_ignore_ascii_case("connect"))
                    || path.parent().is_some_and(|p| p.ends_with("connect"));
                program && fs::read_to_string(path).is_ok_and(|t| t.contains(QUERY))
            });
            assert!(found, "clients/{name} has no connect program with the query");
        }
    }

    #[test]
    fn each_client_has_its_lists() {
        for name in CLIENTS {
            fails::load(name).unwrap();
        }
    }
}
