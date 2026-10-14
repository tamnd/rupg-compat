"""The gate program of spec/21 section 21.4.2, with asyncpg.

asyncpg has no describe command. So the program runs the catalog queries of clients/gate-queries.sql and writes each row with the Python values that asyncpg makes. Then, for each table and view of the schema gate, it prepares a query of the relation and writes the attributes of the statement. asyncpg reads the facts of each type of the result from the catalog with its own introspection query.
"""

import asyncio
import sys

import asyncpg


def queries(path):
    with open(path) as f:
        return [line.strip() for line in f if line.strip() and not line.startswith("--")]


async def main():
    conn = await asyncpg.connect()
    for query in queries(sys.argv[1]):
        stmt = await conn.prepare(query)
        print("query:", query)
        print("columns:", [(a.name, a.type.oid, a.type.name) for a in stmt.get_attributes()])
        for row in await stmt.fetch():
            print("row:", repr(tuple(row)))
        print()
    relations = await conn.fetch(
        "SELECT relname FROM pg_class WHERE relnamespace = 'gate'::regnamespace AND relkind IN ('r', 'v') ORDER BY relname"
    )
    for (name,) in relations:
        stmt = await conn.prepare(f"SELECT * FROM gate.{name} LIMIT 0")
        print("relation:", name)
        for a in stmt.get_attributes():
            print("column:", a.name, a.type.oid, a.type.name, a.type.kind, a.type.schema)
        print()
    await conn.close()


asyncio.run(main())
