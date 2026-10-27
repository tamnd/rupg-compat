"""The connect program of the connect suite, with psycopg2.

psycopg2 uses libpq, so it reads PGSSLMODE, PGSSLROOTCERT and PGCHANNELBINDING. The program writes the row of the query on one line, with one space between the values.
"""

import psycopg2

conn = psycopg2.connect("")
cur = conn.cursor()
cur.execute("SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()")
user, ssl, version = cur.fetchone()
print(user, "t" if ssl else "f", version)
conn.close()
