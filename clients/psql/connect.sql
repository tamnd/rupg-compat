-- The connect program of the connect suite, in psql. rupg-compat sets PGSSLMODE, PGSSLROOTCERT and PGCHANNELBINDING.
-- It writes the row of the query on one line, with one space between the values.
\set ON_ERROR_STOP on
\pset format unaligned
\pset fieldsep ' '
\pset tuples_only on
SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid();
