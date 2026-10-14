"""The gate program of spec/21 section 21.4.2, with psycopg 3.

psycopg has no describe command. So the program runs the catalog queries of clients/gate-queries.sql and writes each row with the Python values that psycopg makes. Then, for each table and view of the schema gate, it writes the columns of cursor.description for a query of the relation and the facts of TypeInfo.fetch for the type of each column.
"""

import sys

import psycopg
from psycopg.types import TypeInfo


def queries(path):
    with open(path) as f:
        return [line.strip() for line in f if line.strip() and not line.startswith("--")]


with psycopg.connect("", autocommit=True) as conn:
    for query in queries(sys.argv[1]):
        cur = conn.execute(query)
        print("query:", query)
        print("columns:", [(c.name, c.type_code) for c in cur.description])
        for row in cur.fetchall():
            print("row:", repr(row))
        print()
    relations = conn.execute(
        "SELECT relname FROM pg_class WHERE relnamespace = 'gate'::regnamespace AND relkind IN ('r', 'v') ORDER BY relname"
    ).fetchall()
    for (name,) in relations:
        cur = conn.execute(f"SELECT * FROM gate.{name} LIMIT 0")
        print("relation:", name)
        for c in cur.description:
            print("column:", c.name, c.type_code, c.type_display, c.internal_size, c.precision, c.scale)
        types = conn.execute(
            "SELECT DISTINCT format_type(atttypid, NULL) FROM pg_attribute WHERE attrelid = %s::regclass AND attnum > 0 ORDER BY 1",
            (f"gate.{name}",),
        ).fetchall()
        for (type_name,) in types:
            info = TypeInfo.fetch(conn, type_name)
            print("type:", type_name, info.name, info.oid, info.array_oid, info.delimiter)
        print()
