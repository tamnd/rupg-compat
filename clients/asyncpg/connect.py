"""The connect program of the connect suite, with asyncpg.

asyncpg reads PGSSLMODE and PGSSLROOTCERT. The program writes the row of the query on one line, with one space between the values.
"""

import asyncio

import asyncpg


async def main():
    conn = await asyncpg.connect()
    user, ssl, version = await conn.fetchrow("SELECT system_user, ssl, version FROM pg_stat_ssl WHERE pid = pg_backend_pid()")
    print(user, "t" if ssl else "f", version)
    await conn.close()


asyncio.run(main())
