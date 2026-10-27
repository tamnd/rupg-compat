# The connect program of the connect suite, with the Ruby pg gem.
# The pg gem uses libpq, so it reads PGSSLMODE, PGSSLROOTCERT and PGCHANNELBINDING. The program writes the row of the query on one line, with one space between the values.
require "pg"

conn = PG.connect
user, ssl, version = conn.exec("SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()").values.first
puts [user, ssl, version].join(" ")
conn.close
