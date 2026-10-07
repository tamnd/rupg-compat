"""The trace scenario of spec/21 section 21.4.2, with asyncpg. Each client runs the same steps."""

import asyncio
import decimal

import asyncpg


async def main():
    # asyncpg reads PGHOST, PGPORT, PGUSER, PGPASSWORD, PGDATABASE and PGSSLMODE.
    conn = await asyncpg.connect()
    # 1. A simple query.
    await conn.fetch("SELECT 1")
    # 2. A query with parameters.
    await conn.fetch("SELECT $1::int4 + 1, $2::text", 41, "abc")
    # 3. The common types.
    await conn.fetch(
        "SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true,"
        " 'x'::text, '\\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date,"
        " ARRAY[1, 2, 3]::int4[], '{\"a\": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid,"
        " NULL::text"
    )
    # 4. A table.
    await conn.execute("CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2))")
    # 5. A transaction that commits.
    async with conn.transaction():
        for row in [(1, "one", "1.50"), (2, "two", "2.50"), (3, "three", "3.50")]:
            await conn.execute("INSERT INTO compat_items VALUES ($1, $2, $3)", row[0], row[1], decimal.Decimal(row[2]))
    # 6. A transaction that rolls back.
    tr = conn.transaction()
    await tr.start()
    await conn.execute("INSERT INTO compat_items VALUES ($1, $2, $3)", 4, "four", decimal.Decimal("4.50"))
    await tr.rollback()
    # 7. The same statement, run six times.
    for _ in range(6):
        await conn.fetch("SELECT id, name, price FROM compat_items WHERE id >= $1 ORDER BY id", 1)
    # 8. An update.
    await conn.execute("UPDATE compat_items SET price = price * 2 WHERE id = $1", 2)
    # 9. An error, then an error in a transaction.
    try:
        await conn.fetch("SELECT 1 / 0")
    except asyncpg.exceptions.DivisionByZeroError:
        pass
    tr = conn.transaction()
    await tr.start()
    try:
        await conn.fetch("SELECT 1 / 0")
    except asyncpg.exceptions.DivisionByZeroError:
        pass
    await tr.rollback()
    # 10. The end.
    await conn.execute("DROP TABLE compat_items")
    await conn.close()


asyncio.run(main())
