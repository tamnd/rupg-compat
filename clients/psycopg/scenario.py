"""The trace scenario of spec/21 section 21.4.2, with psycopg 3. Each client runs the same steps."""

import psycopg

with psycopg.connect("", autocommit=True) as conn:
    # 1. A simple query.
    conn.execute("SELECT 1").fetchall()
    # 2. A query with parameters.
    conn.execute("SELECT %s::int4 + 1, %s::text", (41, "abc")).fetchall()
    # 3. The common types.
    conn.execute(
        "SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true,"
        " 'x'::text, '\\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date,"
        " ARRAY[1, 2, 3]::int4[], '{\"a\": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid,"
        " NULL::text"
    ).fetchall()
    # 4. A table.
    conn.execute("CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2))")
    # 5. A transaction that commits.
    with conn.transaction():
        for row in [(1, "one", "1.50"), (2, "two", "2.50"), (3, "three", "3.50")]:
            conn.execute("INSERT INTO compat_items VALUES (%s, %s, %s)", row)
    # 6. A transaction that rolls back.
    conn.execute("BEGIN")
    conn.execute("INSERT INTO compat_items VALUES (%s, %s, %s)", (4, "four", "4.50"))
    conn.execute("ROLLBACK")
    # 7. The same statement, run six times.
    for _ in range(6):
        conn.execute("SELECT id, name, price FROM compat_items WHERE id >= %s ORDER BY id", (1,)).fetchall()
    # 8. An update.
    conn.execute("UPDATE compat_items SET price = price * 2 WHERE id = %s", (2,))
    # 9. An error, then an error in a transaction.
    try:
        conn.execute("SELECT 1 / 0")
    except psycopg.errors.DivisionByZero:
        pass
    conn.execute("BEGIN")
    try:
        conn.execute("SELECT 1 / 0")
    except psycopg.errors.DivisionByZero:
        pass
    conn.execute("ROLLBACK")
    # 10. The end.
    conn.execute("DROP TABLE compat_items")
