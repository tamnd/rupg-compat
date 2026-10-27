"""The connect program of the connect suite, with psycopg 3.

psycopg uses libpq, so it reads PGSSLMODE, PGSSLROOTCERT and PGCHANNELBINDING. The program writes the row of the query on one line, with one space between the values.
"""

import psycopg

with psycopg.connect("") as conn:
    user, ssl, version = conn.execute("SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()").fetchone()
    print(user, "t" if ssl else "f", version)
