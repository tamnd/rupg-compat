"""The trace scenario of spec/21 section 21.4.2, with psycopg2. Each client runs the same steps."""

import psycopg2
import psycopg2.errors

conn = psycopg2.connect("")
conn.autocommit = True
cur = conn.cursor()
# 1. A simple query.
cur.execute("SELECT 1")
cur.fetchall()
# 2. A query with parameters. psycopg2 puts the values in the text of the query.
cur.execute("SELECT %s::int4 + 1, %s::text", (41, "abc"))
cur.fetchall()
# 3. The common types.
cur.execute(
    "SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true,"
    " 'x'::text, '\\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date,"
    " ARRAY[1, 2, 3]::int4[], '{\"a\": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid,"
    " NULL::text"
)
cur.fetchall()
# 4. A table.
cur.execute("CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2))")
# 5. A transaction that commits.
conn.autocommit = False
for row in [(1, "one", "1.50"), (2, "two", "2.50"), (3, "three", "3.50")]:
    cur.execute("INSERT INTO compat_items VALUES (%s, %s, %s)", row)
conn.commit()
# 6. A transaction that rolls back.
cur.execute("INSERT INTO compat_items VALUES (%s, %s, %s)", (4, "four", "4.50"))
conn.rollback()
conn.autocommit = True
# 7. The same statement, run six times.
for _ in range(6):
    cur.execute("SELECT id, name, price FROM compat_items WHERE id >= %s ORDER BY id", (1,))
    cur.fetchall()
# 8. An update.
cur.execute("UPDATE compat_items SET price = price * 2 WHERE id = %s", (2,))
# 9. An error, then an error in a transaction.
try:
    cur.execute("SELECT 1 / 0")
except psycopg2.errors.DivisionByZero:
    pass
conn.autocommit = False
try:
    cur.execute("SELECT 1 / 0")
except psycopg2.errors.DivisionByZero:
    pass
conn.rollback()
conn.autocommit = True
# 10. The end.
cur.execute("DROP TABLE compat_items")
conn.close()
