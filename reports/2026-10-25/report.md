# rupg-compat report 2026-10-25

This report reads 94 result files in `/private/tmp/work/compat/work/results`. Each number is the count that is equal to the oracle, of the count that the oracle gives. The server column is the server under test, and the oracle column is the oracle against itself. The unstable cases of the oracle are not counted (spec/21 section 21.1). The ratchet checks the count of the oracle less the differences, with each unstable case as no difference, so that a run with more unstable cases does not fail it. A relation that was unstable in the run of the server under test shows as unstable, and the ratchet does not check its rows in that run.

## Pins

| Pin | Release | Commit |
|---|---|---|
| PostgreSQL 14 | 14.24 | `6b3806732b7c5df06bdd0ed150e8a07b4ad62315` |
| PostgreSQL 15 | 15.19 | `2ff1375b5dd8bf09d8cb0e795974528180fd75ca` |
| PostgreSQL 16 | 16.15 | `7d3e000c5961a544302072058a1184e9a588837b` |
| PostgreSQL 17 | 17.11 | `083ac033419f690758508e08c1736089384bbee8` |
| PostgreSQL 18 | 18.6 | `724edf9bde9d356724ad384a2e196edc3c9f80f7` |
| PostgreSQL 19 | 19beta4 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` |
| rupg | | `ead90a6b89c373d927dabcfefbee755b886d9a50` |

| Client | Version |
|---|---|
| activerecord | 8.1.4 |
| asyncpg | 0.32.0 |
| diesel | 2.3.13 |
| django | 6.1.2 |
| drizzle | 0.45.3 |
| ecto | 3.14.0 |
| efcore | 10.0.3 |
| flyway | 13.9.0 |
| golang-migrate | 4.20.1 |
| grafana | 13.2.3 |
| hibernate | 7.4.12.Final |
| lib-pq | 1.12.3 |
| libpq | 19beta4 |
| liquibase | 5.0.4 |
| metabase | 0.64.1 |
| node-postgres | 8.23.1 |
| npgsql | 10.0.3 |
| pg_dump | 19beta4 |
| pgadmin4 | 9.18 |
| pgbouncer | 1.26.0 |
| pgcat | 1.2.0 |
| pgjdbc | 42.7.13 |
| pgx | 5.11.0 |
| postgres-js | 3.4.9 |
| postgrex | 0.22.4 |
| prisma | 7.10.0 |
| psql | 19beta4 |
| psqlodbc | 18.00.0004 |
| psycopg | 3.3.6 |
| psycopg2 | 2.9.13 |
| r2dbc-postgresql | 1.1.3.RELEASE |
| ruby-pg | 1.7.0 |
| sequelize | 6.37.8 |
| sqlalchemy | 2.1.3 |
| sqlx | 0.9.0 |
| supavisor | 2.9.13 |
| superset | 6.1.0 |
| tokio-postgres | 0.7.18 |
| typeorm | 1.1.1 |

## PostgreSQL 19

PostgreSQL 19 is the reference.

| Run | Side | Server | PostgreSQL commit | Started | Seconds | Unstable |
|---|---|---|---|---|---|---|
| catalog | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T01:59:50Z | 2 | 6 |
| catalog | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:40:25Z | 7 | 6 |
| catalog-gate | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:12Z | 1 | 6 |
| catalog-gate | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:30Z | 7 | 6 |
| dump | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T01:59:53Z | 2 | 0 |
| dump | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:40:33Z | 0 | 0 |
| gate | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T01:59:55Z | 77 | 0 |
| gate | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:40:34Z | 55 | 0 |
| isolation | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T01:58:57Z | 51 | 0 |
| isolation | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T03:18:46Z | 2907 | 0 |
| parameters | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T01:59:53Z | 0 | 0 |
| parameters | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:40:33Z | 0 | 0 |
| pipeline | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T01:59:49Z | 0 | 0 |
| pipeline | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:40:23Z | 2 | 0 |
| regress | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T01:58:43Z | 14 | 0 |
| regress | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:27:18Z | 777 | 0 |
| replay-activerecord | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:14Z | 0 | 0 |
| replay-activerecord | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:37Z | 0 | 0 |
| replay-asyncpg | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:14Z | 0 | 0 |
| replay-asyncpg | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:38Z | 3 | 0 |
| replay-diesel | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:14Z | 0 | 0 |
| replay-diesel | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:41Z | 0 | 0 |
| replay-django | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:14Z | 0 | 0 |
| replay-django | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:42Z | 0 | 0 |
| replay-drizzle | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:14Z | 0 | 0 |
| replay-drizzle | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:42Z | 1 | 0 |
| replay-ecto | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:15Z | 0 | 0 |
| replay-ecto | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:43Z | 0 | 0 |
| replay-efcore | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:15Z | 0 | 0 |
| replay-efcore | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:44Z | 1 | 0 |
| replay-flyway | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:15Z | 0 | 0 |
| replay-flyway | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:45Z | 0 | 0 |
| replay-golang-migrate | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:16Z | 0 | 0 |
| replay-golang-migrate | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:46Z | 0 | 0 |
| replay-grafana | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:16Z | 0 | 0 |
| replay-grafana | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:46Z | 0 | 0 |
| replay-hibernate | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:16Z | 0 | 0 |
| replay-hibernate | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:47Z | 0 | 0 |
| replay-lib-pq | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:16Z | 0 | 0 |
| replay-lib-pq | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:47Z | 0 | 0 |
| replay-libpq | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:16Z | 0 | 0 |
| replay-libpq | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:48Z | 0 | 0 |
| replay-liquibase | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:16Z | 0 | 0 |
| replay-liquibase | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:48Z | 2 | 0 |
| replay-metabase | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:17Z | 0 | 0 |
| replay-metabase | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:51Z | 0 | 0 |
| replay-node-postgres | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:17Z | 0 | 0 |
| replay-node-postgres | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:51Z | 0 | 0 |
| replay-npgsql | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:17Z | 0 | 0 |
| replay-npgsql | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:51Z | 0 | 0 |
| replay-pg_dump | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:17Z | 0 | 0 |
| replay-pg_dump | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:52Z | 2 | 0 |
| replay-pgadmin4 | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:18Z | 0 | 0 |
| replay-pgadmin4 | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:54Z | 1 | 0 |
| replay-pgbouncer | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:18Z | 0 | 0 |
| replay-pgbouncer | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:41:55Z | 9 | 0 |
| replay-pgcat | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:19Z | 1 | 0 |
| replay-pgcat | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:05Z | 10 | 0 |
| replay-pgjdbc | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:20Z | 0 | 0 |
| replay-pgjdbc | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:15Z | 0 | 0 |
| replay-pgx | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:20Z | 0 | 0 |
| replay-pgx | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:16Z | 0 | 0 |
| replay-postgres-js | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:21Z | 0 | 0 |
| replay-postgres-js | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:16Z | 0 | 0 |
| replay-postgrex | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:21Z | 0 | 0 |
| replay-postgrex | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:16Z | 0 | 0 |
| replay-prisma | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:21Z | 0 | 0 |
| replay-prisma | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:17Z | 0 | 0 |
| replay-psql | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:21Z | 0 | 0 |
| replay-psql | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:17Z | 0 | 0 |
| replay-psqlodbc | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:21Z | 0 | 0 |
| replay-psqlodbc | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:18Z | 0 | 0 |
| replay-psycopg | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:21Z | 0 | 0 |
| replay-psycopg | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:18Z | 0 | 0 |
| replay-psycopg2 | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:22Z | 0 | 0 |
| replay-psycopg2 | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:18Z | 0 | 0 |
| replay-r2dbc-postgresql | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:22Z | 0 | 0 |
| replay-r2dbc-postgresql | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:19Z | 0 | 0 |
| replay-ruby-pg | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:22Z | 0 | 0 |
| replay-ruby-pg | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:19Z | 0 | 0 |
| replay-sequelize | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:22Z | 0 | 0 |
| replay-sequelize | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:19Z | 0 | 0 |
| replay-sqlalchemy | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:22Z | 0 | 0 |
| replay-sqlalchemy | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:20Z | 0 | 0 |
| replay-sqlx | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:22Z | 0 | 0 |
| replay-sqlx | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:20Z | 0 | 0 |
| replay-supavisor | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:22Z | 0 | 0 |
| replay-supavisor | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:21Z | 11 | 0 |
| replay-superset | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:23Z | 0 | 0 |
| replay-superset | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:32Z | 2 | 0 |
| replay-tokio-postgres | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:23Z | 0 | 0 |
| replay-tokio-postgres | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:35Z | 0 | 0 |
| replay-typeorm | oracle | 127.0.0.1:54319 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:01:24Z | 0 | 0 |
| replay-typeorm | server | 127.0.0.1:55480 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` | 2026-10-25T02:42:35Z | 2 | 0 |

### L1 Connect

| Row | Unit | Server | Oracle |
|---|---|---|---|
| libpq_pipeline | tests | 24 of 24 (100.0%) | 24 of 24 (100.0%) |
| trace activerecord | messages | 829 of 829 (100.0%) | 829 of 829 (100.0%) |
| trace asyncpg | messages | 140 of 140 (100.0%) | 140 of 140 (100.0%) |
| trace diesel | messages | 449 of 449 (100.0%) | 449 of 449 (100.0%) |
| trace django | messages | 112 of 112 (100.0%) | 112 of 112 (100.0%) |
| trace drizzle | messages | 425 of 431 (98.6%) | 431 of 431 (100.0%) |
| trace ecto | messages | 633 of 633 (100.0%) | 633 of 633 (100.0%) |
| trace efcore | messages | 2296 of 4078 (56.3%) | 2298 of 4078 (56.4%) |
| trace flyway | messages | 1289 of 1289 (100.0%) | 1289 of 1289 (100.0%) |
| trace golang-migrate | messages | 172 of 172 (100.0%) | 172 of 172 (100.0%) |
| trace grafana | messages | 176 of 176 (100.0%) | 176 of 176 (100.0%) |
| trace hibernate | messages | 402 of 402 (100.0%) | 402 of 402 (100.0%) |
| trace lib-pq | messages | 152 of 152 (100.0%) | 152 of 152 (100.0%) |
| trace libpq | messages | 124 of 124 (100.0%) | 124 of 124 (100.0%) |
| trace liquibase | messages | 1950 of 1978 (98.6%) | 1950 of 1978 (98.6%) |
| trace metabase | messages | 593 of 593 (100.0%) | 593 of 593 (100.0%) |
| trace node-postgres | messages | 123 of 123 (100.0%) | 123 of 123 (100.0%) |
| trace npgsql | messages | 320 of 320 (100.0%) | 320 of 320 (100.0%) |
| trace pg_dump | messages | 20798 of 22596 (92.0%) | 18990 of 22596 (84.0%) |
| trace pgadmin4 | messages | 384 of 392 (98.0%) | 392 of 392 (100.0%) |
| trace pgbouncer | messages | 1385 of 1385 (100.0%) | 1385 of 1385 (100.0%) |
| trace pgcat | messages | 1381 of 1381 (100.0%) | 1381 of 1381 (100.0%) |
| trace pgjdbc | messages | 146 of 146 (100.0%) | 146 of 146 (100.0%) |
| trace pgx | messages | 133 of 133 (100.0%) | 133 of 133 (100.0%) |
| trace postgres-js | messages | 445 of 445 (100.0%) | 445 of 445 (100.0%) |
| trace postgrex | messages | 359 of 359 (100.0%) | 359 of 359 (100.0%) |
| trace prisma | messages | 584 of 585 (99.8%) | 584 of 585 (99.8%) |
| trace psql | messages | 126 of 126 (100.0%) | 126 of 126 (100.0%) |
| trace psqlodbc | messages | 162 of 162 (100.0%) | 162 of 162 (100.0%) |
| trace psycopg | messages | 131 of 131 (100.0%) | 131 of 131 (100.0%) |
| trace psycopg2 | messages | 99 of 99 (100.0%) | 99 of 99 (100.0%) |
| trace r2dbc-postgresql | messages | 139 of 139 (100.0%) | 139 of 139 (100.0%) |
| trace ruby-pg | messages | 124 of 124 (100.0%) | 124 of 124 (100.0%) |
| trace sequelize | messages | 534 of 538 (99.3%) | 538 of 538 (100.0%) |
| trace sqlalchemy | messages | 412 of 412 (100.0%) | 412 of 412 (100.0%) |
| trace sqlx | messages | 146 of 146 (100.0%) | 146 of 146 (100.0%) |
| trace supavisor | messages | 1629 of 1629 (100.0%) | 1629 of 1629 (100.0%) |
| trace superset | messages | 1569 of 1569 (100.0%) | 1569 of 1569 (100.0%) |
| trace tokio-postgres | messages | 214 of 214 (100.0%) | 214 of 214 (100.0%) |
| trace typeorm | messages | 240 of 240 (100.0%) | 240 of 240 (100.0%) |

### L2 Introspect

| Row | Unit | Server | Oracle |
|---|---|---|---|
| catalog relations | relations | 184 of 209 (88.0%) | 209 of 209 (100.0%) |
| catalog relations with the gate schema | relations | 184 of 209 (88.0%) | 209 of 209 (100.0%) |
| gate G1 | clients | 6 of 8 (75.0%) | 6 of 8 (75.0%) |
| pg_dump round trip | checks | 6 of 7 (85.7%) | 7 of 7 (100.0%) |

The rows of each relation:

| Relation | Server | Oracle |
|---|---|---|
| information_schema._pg_foreign_data_wrappers | 0 of 0 | 0 of 0 |
| information_schema._pg_foreign_data_wrappers with the gate schema | 0 of 0 | 0 of 0 |
| information_schema._pg_foreign_servers | 0 of 0 | 0 of 0 |
| information_schema._pg_foreign_servers with the gate schema | 0 of 0 | 0 of 0 |
| information_schema._pg_foreign_table_columns | 0 of 0 | 0 of 0 |
| information_schema._pg_foreign_table_columns with the gate schema | 0 of 0 | 0 of 0 |
| information_schema._pg_foreign_tables | 0 of 0 | 0 of 0 |
| information_schema._pg_foreign_tables with the gate schema | 0 of 0 | 0 of 0 |
| information_schema._pg_user_mappings | 0 of 0 | 0 of 0 |
| information_schema._pg_user_mappings with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.administrable_role_authorizations | 0 of 0 | 0 of 0 |
| information_schema.administrable_role_authorizations with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.applicable_roles | 4 of 4 (100.0%) | 4 of 4 (100.0%) |
| information_schema.applicable_roles with the gate schema | 4 of 4 (100.0%) | 4 of 4 (100.0%) |
| information_schema.attributes | 0 of 0 | 0 of 0 |
| information_schema.attributes with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.character_sets | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| information_schema.character_sets with the gate schema | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| information_schema.check_constraint_routine_usage | 0 of 0 | 0 of 0 |
| information_schema.check_constraint_routine_usage with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.check_constraints | 84 of 84 (100.0%) | 84 of 84 (100.0%) |
| information_schema.check_constraints with the gate schema | 104 of 104 (100.0%) | 104 of 104 (100.0%) |
| information_schema.collation_character_set_applicability | 858 of 1079 (79.5%) | 1079 of 1079 (100.0%) |
| information_schema.collation_character_set_applicability with the gate schema | 858 of 1079 (79.5%) | 1079 of 1079 (100.0%) |
| information_schema.collations | 858 of 1079 (79.5%) | 1079 of 1079 (100.0%) |
| information_schema.collations with the gate schema | 858 of 1079 (79.5%) | 1079 of 1079 (100.0%) |
| information_schema.column_column_usage | 0 of 0 | 0 of 0 |
| information_schema.column_column_usage with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.column_domain_usage | 683 of 683 (100.0%) | 683 of 683 (100.0%) |
| information_schema.column_domain_usage with the gate schema | 683 of 683 (100.0%) | 683 of 683 (100.0%) |
| information_schema.column_options | 0 of 0 | 0 of 0 |
| information_schema.column_options with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.column_privileges | 10958 of 10958 (100.0%) | 10958 of 10958 (100.0%) |
| information_schema.column_privileges with the gate schema | 11166 of 11166 (100.0%) | 11166 of 11166 (100.0%) |
| information_schema.column_udt_usage | 2217 of 2217 (100.0%) | 2217 of 2217 (100.0%) |
| information_schema.column_udt_usage with the gate schema | 2269 of 2269 (100.0%) | 2269 of 2269 (100.0%) |
| information_schema.columns | 2217 of 2217 (100.0%) | 2217 of 2217 (100.0%) |
| information_schema.columns with the gate schema | 2269 of 2269 (100.0%) | 2269 of 2269 (100.0%) |
| information_schema.constraint_column_usage | 263 of 263 (100.0%) | 263 of 263 (100.0%) |
| information_schema.constraint_column_usage with the gate schema | 293 of 293 (100.0%) | 293 of 293 (100.0%) |
| information_schema.constraint_table_usage | 110 of 110 (100.0%) | 110 of 110 (100.0%) |
| information_schema.constraint_table_usage with the gate schema | 119 of 119 (100.0%) | 119 of 119 (100.0%) |
| information_schema.data_type_privileges | 12411 of 12411 (100.0%) | 12411 of 12411 (100.0%) |
| information_schema.data_type_privileges with the gate schema | 12463 of 12463 (100.0%) | 12463 of 12463 (100.0%) |
| information_schema.domain_constraints | 2 of 2 (100.0%) | 2 of 2 (100.0%) |
| information_schema.domain_constraints with the gate schema | 2 of 2 (100.0%) | 2 of 2 (100.0%) |
| information_schema.domain_udt_usage | 5 of 5 (100.0%) | 5 of 5 (100.0%) |
| information_schema.domain_udt_usage with the gate schema | 5 of 5 (100.0%) | 5 of 5 (100.0%) |
| information_schema.domains | 5 of 5 (100.0%) | 5 of 5 (100.0%) |
| information_schema.domains with the gate schema | 5 of 5 (100.0%) | 5 of 5 (100.0%) |
| information_schema.element_types | 278 of 278 (100.0%) | 278 of 278 (100.0%) |
| information_schema.element_types with the gate schema | 279 of 279 (100.0%) | 279 of 279 (100.0%) |
| information_schema.enabled_roles | 17 of 17 (100.0%) | 17 of 17 (100.0%) |
| information_schema.enabled_roles with the gate schema | 17 of 17 (100.0%) | 17 of 17 (100.0%) |
| information_schema.foreign_data_wrapper_options | 0 of 0 | 0 of 0 |
| information_schema.foreign_data_wrapper_options with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.foreign_data_wrappers | 0 of 0 | 0 of 0 |
| information_schema.foreign_data_wrappers with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.foreign_server_options | 0 of 0 | 0 of 0 |
| information_schema.foreign_server_options with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.foreign_servers | 0 of 0 | 0 of 0 |
| information_schema.foreign_servers with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.foreign_table_options | 0 of 0 | 0 of 0 |
| information_schema.foreign_table_options with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.foreign_tables | 0 of 0 | 0 of 0 |
| information_schema.foreign_tables with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.information_schema_catalog_name | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| information_schema.information_schema_catalog_name with the gate schema | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| information_schema.key_column_usage | 181 of 181 (100.0%) | 181 of 181 (100.0%) |
| information_schema.key_column_usage with the gate schema | 191 of 191 (100.0%) | 191 of 191 (100.0%) |
| information_schema.parameters | 6759 of 6759 (100.0%) | 6759 of 6759 (100.0%) |
| information_schema.parameters with the gate schema | 6759 of 6759 (100.0%) | 6759 of 6759 (100.0%) |
| information_schema.referential_constraints | 0 of 0 | 0 of 0 |
| information_schema.referential_constraints with the gate schema | 3 of 3 (100.0%) | 3 of 3 (100.0%) |
| information_schema.role_column_grants | 10958 of 10958 (100.0%) | 10958 of 10958 (100.0%) |
| information_schema.role_column_grants with the gate schema | 11166 of 11166 (100.0%) | 11166 of 11166 (100.0%) |
| information_schema.role_routine_grants | 6809 of 6809 (100.0%) | 6809 of 6809 (100.0%) |
| information_schema.role_routine_grants with the gate schema | 6809 of 6809 (100.0%) | 6809 of 6809 (100.0%) |
| information_schema.role_table_grants | 1734 of 1734 (100.0%) | 1734 of 1734 (100.0%) |
| information_schema.role_table_grants with the gate schema | 1776 of 1776 (100.0%) | 1776 of 1776 (100.0%) |
| information_schema.role_udt_grants | 438 of 438 (100.0%) | 438 of 438 (100.0%) |
| information_schema.role_udt_grants with the gate schema | 450 of 450 (100.0%) | 450 of 450 (100.0%) |
| information_schema.role_usage_grants | 868 of 1089 (79.7%) | 1089 of 1089 (100.0%) |
| information_schema.role_usage_grants with the gate schema | 868 of 1089 (79.7%) | 1089 of 1089 (100.0%) |
| information_schema.routine_column_usage | 0 of 0 | 0 of 0 |
| information_schema.routine_column_usage with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.routine_privileges | 6809 of 6809 (100.0%) | 6809 of 6809 (100.0%) |
| information_schema.routine_privileges with the gate schema | 6809 of 6809 (100.0%) | 6809 of 6809 (100.0%) |
| information_schema.routine_routine_usage | 2 of 2 (100.0%) | 2 of 2 (100.0%) |
| information_schema.routine_routine_usage with the gate schema | 2 of 2 (100.0%) | 2 of 2 (100.0%) |
| information_schema.routine_sequence_usage | 0 of 0 | 0 of 0 |
| information_schema.routine_sequence_usage with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.routine_table_usage | 0 of 0 | 0 of 0 |
| information_schema.routine_table_usage with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.routines | 3430 of 3430 (100.0%) | 3430 of 3430 (100.0%) |
| information_schema.routines with the gate schema | 3430 of 3430 (100.0%) | 3430 of 3430 (100.0%) |
| information_schema.schemata | 4 of 4 (100.0%) | 4 of 4 (100.0%) |
| information_schema.schemata with the gate schema | 5 of 5 (100.0%) | 5 of 5 (100.0%) |
| information_schema.sequences | 0 of 0 | 0 of 0 |
| information_schema.sequences with the gate schema | 3 of 3 (100.0%) | 3 of 3 (100.0%) |
| information_schema.table_constraints | 192 of 192 (100.0%) | 192 of 192 (100.0%) |
| information_schema.table_constraints with the gate schema | 221 of 221 (100.0%) | 221 of 221 (100.0%) |
| information_schema.table_privileges | 1734 of 1734 (100.0%) | 1734 of 1734 (100.0%) |
| information_schema.table_privileges with the gate schema | 1776 of 1776 (100.0%) | 1776 of 1776 (100.0%) |
| information_schema.tables | 219 of 219 (100.0%) | 219 of 219 (100.0%) |
| information_schema.tables with the gate schema | 225 of 225 (100.0%) | 225 of 225 (100.0%) |
| information_schema.transforms | 0 of 0 | 0 of 0 |
| information_schema.transforms with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.triggered_update_columns | 0 of 0 | 0 of 0 |
| information_schema.triggered_update_columns with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.triggers | 0 of 0 | 0 of 0 |
| information_schema.triggers with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.udt_privileges | 438 of 438 (100.0%) | 438 of 438 (100.0%) |
| information_schema.udt_privileges with the gate schema | 450 of 450 (100.0%) | 450 of 450 (100.0%) |
| information_schema.usage_privileges | 868 of 1089 (79.7%) | 1089 of 1089 (100.0%) |
| information_schema.usage_privileges with the gate schema | 868 of 1089 (79.7%) | 1089 of 1089 (100.0%) |
| information_schema.user_defined_types | 0 of 0 | 0 of 0 |
| information_schema.user_defined_types with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.user_mapping_options | 0 of 0 | 0 of 0 |
| information_schema.user_mapping_options with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.user_mappings | 0 of 0 | 0 of 0 |
| information_schema.user_mappings with the gate schema | 0 of 0 | 0 of 0 |
| information_schema.view_column_usage | 292 of 292 (100.0%) | 292 of 292 (100.0%) |
| information_schema.view_column_usage with the gate schema | 297 of 297 (100.0%) | 297 of 297 (100.0%) |
| information_schema.view_routine_usage | 31 of 31 (100.0%) | 31 of 31 (100.0%) |
| information_schema.view_routine_usage with the gate schema | 31 of 31 (100.0%) | 31 of 31 (100.0%) |
| information_schema.view_table_usage | 42 of 42 (100.0%) | 42 of 42 (100.0%) |
| information_schema.view_table_usage with the gate schema | 44 of 44 (100.0%) | 44 of 44 (100.0%) |
| information_schema.views | 151 of 151 (100.0%) | 151 of 151 (100.0%) |
| information_schema.views with the gate schema | 152 of 152 (100.0%) | 152 of 152 (100.0%) |
| pg_catalog.pg_aggregate | 163 of 163 (100.0%) | 163 of 163 (100.0%) |
| pg_catalog.pg_aggregate with the gate schema | 163 of 163 (100.0%) | 163 of 163 (100.0%) |
| pg_catalog.pg_aios | 0 of 0 | 0 of 0 |
| pg_catalog.pg_aios with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_am | 7 of 7 (100.0%) | 7 of 7 (100.0%) |
| pg_catalog.pg_am with the gate schema | 7 of 7 (100.0%) | 7 of 7 (100.0%) |
| pg_catalog.pg_amop | 951 of 951 (100.0%) | 951 of 951 (100.0%) |
| pg_catalog.pg_amop with the gate schema | 951 of 951 (100.0%) | 951 of 951 (100.0%) |
| pg_catalog.pg_amproc | 720 of 720 (100.0%) | 720 of 720 (100.0%) |
| pg_catalog.pg_amproc with the gate schema | 720 of 720 (100.0%) | 720 of 720 (100.0%) |
| pg_catalog.pg_attrdef | 0 of 0 | 0 of 0 |
| pg_catalog.pg_attrdef with the gate schema | 7 of 7 (100.0%) | 7 of 7 (100.0%) |
| pg_catalog.pg_attribute | 3259 of 3259 (100.0%) | 3259 of 3259 (100.0%) |
| pg_catalog.pg_attribute with the gate schema | 3424 of 3424 (100.0%) | 3424 of 3424 (100.0%) |
| pg_catalog.pg_auth_members | 3 of 3 (100.0%) | 3 of 3 (100.0%) |
| pg_catalog.pg_auth_members with the gate schema | 3 of 3 (100.0%) | 3 of 3 (100.0%) |
| pg_catalog.pg_authid | 17 of 17 (100.0%) | 17 of 17 (100.0%) |
| pg_catalog.pg_authid with the gate schema | 17 of 17 (100.0%) | 17 of 17 (100.0%) |
| pg_catalog.pg_available_extension_versions | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_available_extension_versions with the gate schema | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_available_extensions | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_available_extensions with the gate schema | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_backend_memory_contexts | unstable | unstable |
| pg_catalog.pg_backend_memory_contexts with the gate schema | unstable | unstable |
| pg_catalog.pg_cast | 249 of 249 (100.0%) | 249 of 249 (100.0%) |
| pg_catalog.pg_cast with the gate schema | 249 of 249 (100.0%) | 249 of 249 (100.0%) |
| pg_catalog.pg_class | 421 of 421 (100.0%) | 421 of 421 (100.0%) |
| pg_catalog.pg_class with the gate schema | 448 of 448 (100.0%) | 448 of 448 (100.0%) |
| pg_catalog.pg_collation | 6 of 1286 (0.5%) | 1286 of 1286 (100.0%) |
| pg_catalog.pg_collation with the gate schema | 6 of 1286 (0.5%) | 1286 of 1286 (100.0%) |
| pg_catalog.pg_config | 23 of 23 (100.0%) | 23 of 23 (100.0%) |
| pg_catalog.pg_config with the gate schema | 23 of 23 (100.0%) | 23 of 23 (100.0%) |
| pg_catalog.pg_constraint | 194 of 194 (100.0%) | 194 of 194 (100.0%) |
| pg_catalog.pg_constraint with the gate schema | 223 of 223 (100.0%) | 223 of 223 (100.0%) |
| pg_catalog.pg_conversion | 98 of 98 (100.0%) | 98 of 98 (100.0%) |
| pg_catalog.pg_conversion with the gate schema | 98 of 98 (100.0%) | 98 of 98 (100.0%) |
| pg_catalog.pg_cursors | 0 of 0 | 0 of 0 |
| pg_catalog.pg_cursors with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_database | 10 of 11 (90.9%) | 7 of 7 (100.0%) |
| pg_catalog.pg_database with the gate schema | 10 of 11 (90.9%) | 10 of 10 (100.0%) |
| pg_catalog.pg_db_role_setting | 2 of 2 (100.0%) | 2 of 2 (100.0%) |
| pg_catalog.pg_db_role_setting with the gate schema | 2 of 2 (100.0%) | 2 of 2 (100.0%) |
| pg_catalog.pg_default_acl | 0 of 0 | 0 of 0 |
| pg_catalog.pg_default_acl with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_depend | 1770 of 1770 (100.0%) | 1770 of 1770 (100.0%) |
| pg_catalog.pg_depend with the gate schema | 1887 of 1887 (100.0%) | 1887 of 1887 (100.0%) |
| pg_catalog.pg_description | 5352 of 5407 (99.0%) | 5407 of 5407 (100.0%) |
| pg_catalog.pg_description with the gate schema | 5356 of 5411 (99.0%) | 5411 of 5411 (100.0%) |
| pg_catalog.pg_dsm_registry_allocations | 0 of 0 | 0 of 0 |
| pg_catalog.pg_dsm_registry_allocations with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_enum | 0 of 0 | 0 of 0 |
| pg_catalog.pg_enum with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_event_trigger | 0 of 0 | 0 of 0 |
| pg_catalog.pg_event_trigger with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_extension | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_extension with the gate schema | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_file_settings | 0 of 31 (0.0%) | 31 of 31 (100.0%) |
| pg_catalog.pg_file_settings with the gate schema | 0 of 31 (0.0%) | 31 of 31 (100.0%) |
| pg_catalog.pg_foreign_data_wrapper | 0 of 0 | 0 of 0 |
| pg_catalog.pg_foreign_data_wrapper with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_foreign_server | 0 of 0 | 0 of 0 |
| pg_catalog.pg_foreign_server with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_foreign_table | 0 of 0 | 0 of 0 |
| pg_catalog.pg_foreign_table with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_group | 16 of 16 (100.0%) | 16 of 16 (100.0%) |
| pg_catalog.pg_group with the gate schema | 16 of 16 (100.0%) | 16 of 16 (100.0%) |
| pg_catalog.pg_hba_file_rules | 3 of 3 (100.0%) | 3 of 3 (100.0%) |
| pg_catalog.pg_hba_file_rules with the gate schema | 3 of 3 (100.0%) | 3 of 3 (100.0%) |
| pg_catalog.pg_ident_file_mappings | 0 of 0 | 0 of 0 |
| pg_catalog.pg_ident_file_mappings with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_index | 163 of 163 (100.0%) | 163 of 163 (100.0%) |
| pg_catalog.pg_index with the gate schema | 177 of 177 (100.0%) | 177 of 177 (100.0%) |
| pg_catalog.pg_indexes | 124 of 124 (100.0%) | 124 of 124 (100.0%) |
| pg_catalog.pg_indexes with the gate schema | 134 of 134 (100.0%) | 134 of 134 (100.0%) |
| pg_catalog.pg_inherits | 0 of 0 | 0 of 0 |
| pg_catalog.pg_inherits with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_init_privs | 241 of 241 (100.0%) | 241 of 241 (100.0%) |
| pg_catalog.pg_init_privs with the gate schema | 241 of 241 (100.0%) | 241 of 241 (100.0%) |
| pg_catalog.pg_language | 4 of 4 (100.0%) | 4 of 4 (100.0%) |
| pg_catalog.pg_language with the gate schema | 4 of 4 (100.0%) | 4 of 4 (100.0%) |
| pg_catalog.pg_largeobject | 0 of 0 | 0 of 0 |
| pg_catalog.pg_largeobject with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_largeobject_metadata | 0 of 0 | 0 of 0 |
| pg_catalog.pg_largeobject_metadata with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_locks | unstable | unstable |
| pg_catalog.pg_locks with the gate schema | unstable | unstable |
| pg_catalog.pg_matviews | 0 of 0 | 0 of 0 |
| pg_catalog.pg_matviews with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_namespace | 4 of 4 (100.0%) | 4 of 4 (100.0%) |
| pg_catalog.pg_namespace with the gate schema | 5 of 5 (100.0%) | 5 of 5 (100.0%) |
| pg_catalog.pg_opclass | 179 of 179 (100.0%) | 179 of 179 (100.0%) |
| pg_catalog.pg_opclass with the gate schema | 179 of 179 (100.0%) | 179 of 179 (100.0%) |
| pg_catalog.pg_operator | 805 of 805 (100.0%) | 805 of 805 (100.0%) |
| pg_catalog.pg_operator with the gate schema | 805 of 805 (100.0%) | 805 of 805 (100.0%) |
| pg_catalog.pg_opfamily | 148 of 148 (100.0%) | 148 of 148 (100.0%) |
| pg_catalog.pg_opfamily with the gate schema | 148 of 148 (100.0%) | 148 of 148 (100.0%) |
| pg_catalog.pg_parameter_acl | 0 of 0 | 0 of 0 |
| pg_catalog.pg_parameter_acl with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_partitioned_table | 0 of 0 | 0 of 0 |
| pg_catalog.pg_partitioned_table with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_policies | 0 of 0 | 0 of 0 |
| pg_catalog.pg_policies with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_policy | 0 of 0 | 0 of 0 |
| pg_catalog.pg_policy with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_prepared_statements | 0 of 0 | 0 of 0 |
| pg_catalog.pg_prepared_statements with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_prepared_xacts | 0 of 0 | 0 of 0 |
| pg_catalog.pg_prepared_xacts with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_proc | 3430 of 3430 (100.0%) | 3430 of 3430 (100.0%) |
| pg_catalog.pg_proc with the gate schema | 3430 of 3430 (100.0%) | 3430 of 3430 (100.0%) |
| pg_catalog.pg_publication | 0 of 0 | 0 of 0 |
| pg_catalog.pg_publication with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_publication_namespace | 0 of 0 | 0 of 0 |
| pg_catalog.pg_publication_namespace with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_publication_rel | 0 of 0 | 0 of 0 |
| pg_catalog.pg_publication_rel with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_publication_sequences | 0 of 0 | 0 of 0 |
| pg_catalog.pg_publication_sequences with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_publication_tables | 0 of 0 | 0 of 0 |
| pg_catalog.pg_publication_tables with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_range | 6 of 6 (100.0%) | 6 of 6 (100.0%) |
| pg_catalog.pg_range with the gate schema | 6 of 6 (100.0%) | 6 of 6 (100.0%) |
| pg_catalog.pg_replication_origin | 0 of 0 | 0 of 0 |
| pg_catalog.pg_replication_origin with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_replication_origin_status | 0 of 0 | 0 of 0 |
| pg_catalog.pg_replication_origin_status with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_replication_slots | 0 of 0 | 0 of 0 |
| pg_catalog.pg_replication_slots with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_rewrite | 153 of 153 (100.0%) | 153 of 153 (100.0%) |
| pg_catalog.pg_rewrite with the gate schema | 154 of 154 (100.0%) | 154 of 154 (100.0%) |
| pg_catalog.pg_roles | 17 of 17 (100.0%) | 17 of 17 (100.0%) |
| pg_catalog.pg_roles with the gate schema | 17 of 17 (100.0%) | 17 of 17 (100.0%) |
| pg_catalog.pg_rules | 2 of 2 (100.0%) | 2 of 2 (100.0%) |
| pg_catalog.pg_rules with the gate schema | 2 of 2 (100.0%) | 2 of 2 (100.0%) |
| pg_catalog.pg_seclabel | 0 of 0 | 0 of 0 |
| pg_catalog.pg_seclabel with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_seclabels | 0 of 0 | 0 of 0 |
| pg_catalog.pg_seclabels with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_sequence | 0 of 0 | 0 of 0 |
| pg_catalog.pg_sequence with the gate schema | 3 of 3 (100.0%) | 3 of 3 (100.0%) |
| pg_catalog.pg_sequences | 0 of 0 | 0 of 0 |
| pg_catalog.pg_sequences with the gate schema | 3 of 3 (100.0%) | 3 of 3 (100.0%) |
| pg_catalog.pg_settings | 398 of 419 (95.0%) | 419 of 419 (100.0%) |
| pg_catalog.pg_settings with the gate schema | 398 of 419 (95.0%) | 419 of 419 (100.0%) |
| pg_catalog.pg_shadow | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_shadow with the gate schema | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_shdepend | 0 of 0 | 0 of 0 |
| pg_catalog.pg_shdepend with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_shdescription | 3 of 3 (100.0%) | 3 of 3 (100.0%) |
| pg_catalog.pg_shdescription with the gate schema | 3 of 3 (100.0%) | 3 of 3 (100.0%) |
| pg_catalog.pg_shmem_allocations | 0 of 79 (0.0%) | 79 of 79 (100.0%) |
| pg_catalog.pg_shmem_allocations with the gate schema | 0 of 79 (0.0%) | 79 of 79 (100.0%) |
| pg_catalog.pg_shmem_allocations_numa | 0 of 0 | 0 of 0 |
| pg_catalog.pg_shmem_allocations_numa with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_shseclabel | 0 of 0 | 0 of 0 |
| pg_catalog.pg_shseclabel with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_activity | 1 of 8 (12.5%) | 9 of 9 (100.0%) |
| pg_catalog.pg_stat_activity with the gate schema | 1 of 8 (12.5%) | 8 of 8 (100.0%) |
| pg_catalog.pg_stat_all_indexes | unstable | unstable |
| pg_catalog.pg_stat_all_indexes with the gate schema | unstable | unstable |
| pg_catalog.pg_stat_all_tables | unstable | unstable |
| pg_catalog.pg_stat_all_tables with the gate schema | unstable | unstable |
| pg_catalog.pg_stat_archiver | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_stat_archiver with the gate schema | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_stat_autovacuum_scores | 107 of 107 (100.0%) | 107 of 107 (100.0%) |
| pg_catalog.pg_stat_autovacuum_scores with the gate schema | 116 of 116 (100.0%) | 116 of 116 (100.0%) |
| pg_catalog.pg_stat_bgwriter | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_stat_bgwriter with the gate schema | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_stat_checkpointer | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_stat_checkpointer with the gate schema | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_stat_database | 10 of 12 (83.3%) | 8 of 8 (100.0%) |
| pg_catalog.pg_stat_database with the gate schema | 10 of 12 (83.3%) | 11 of 11 (100.0%) |
| pg_catalog.pg_stat_database_conflicts | 10 of 11 (90.9%) | 7 of 7 (100.0%) |
| pg_catalog.pg_stat_database_conflicts with the gate schema | 10 of 11 (90.9%) | 10 of 10 (100.0%) |
| pg_catalog.pg_stat_gssapi | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_stat_gssapi with the gate schema | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_stat_io | 79 of 79 (100.0%) | 79 of 79 (100.0%) |
| pg_catalog.pg_stat_io with the gate schema | 79 of 79 (100.0%) | 79 of 79 (100.0%) |
| pg_catalog.pg_stat_lock | 12 of 12 (100.0%) | 12 of 12 (100.0%) |
| pg_catalog.pg_stat_lock with the gate schema | 12 of 12 (100.0%) | 12 of 12 (100.0%) |
| pg_catalog.pg_stat_progress_analyze | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_progress_analyze with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_progress_basebackup | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_progress_basebackup with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_progress_cluster | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_progress_cluster with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_progress_copy | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_progress_copy with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_progress_create_index | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_progress_create_index with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_progress_repack | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_progress_repack with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_progress_vacuum | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_progress_vacuum with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_recovery | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_recovery with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_recovery_prefetch | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_stat_recovery_prefetch with the gate schema | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_stat_replication | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_replication with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_replication_slots | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_replication_slots with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_slru | 8 of 8 (100.0%) | 8 of 8 (100.0%) |
| pg_catalog.pg_stat_slru with the gate schema | 8 of 8 (100.0%) | 8 of 8 (100.0%) |
| pg_catalog.pg_stat_ssl | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_stat_ssl with the gate schema | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_stat_subscription | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_subscription with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_subscription_stats | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_subscription_stats with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_sys_indexes | unstable | unstable |
| pg_catalog.pg_stat_sys_indexes with the gate schema | unstable | unstable |
| pg_catalog.pg_stat_sys_tables | unstable | unstable |
| pg_catalog.pg_stat_sys_tables with the gate schema | unstable | unstable |
| pg_catalog.pg_stat_user_functions | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_user_functions with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_user_indexes | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_user_indexes with the gate schema | 10 of 10 (100.0%) | 10 of 10 (100.0%) |
| pg_catalog.pg_stat_user_tables | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_user_tables with the gate schema | 5 of 5 (100.0%) | 5 of 5 (100.0%) |
| pg_catalog.pg_stat_wal | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_stat_wal with the gate schema | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_stat_wal_receiver | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_wal_receiver with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_xact_all_tables | 107 of 107 (100.0%) | 107 of 107 (100.0%) |
| pg_catalog.pg_stat_xact_all_tables with the gate schema | 116 of 116 (100.0%) | 116 of 116 (100.0%) |
| pg_catalog.pg_stat_xact_sys_tables | 107 of 107 (100.0%) | 107 of 107 (100.0%) |
| pg_catalog.pg_stat_xact_sys_tables with the gate schema | 111 of 111 (100.0%) | 111 of 111 (100.0%) |
| pg_catalog.pg_stat_xact_user_functions | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_xact_user_functions with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_xact_user_tables | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stat_xact_user_tables with the gate schema | 5 of 5 (100.0%) | 5 of 5 (100.0%) |
| pg_catalog.pg_statio_all_indexes | 163 of 163 (100.0%) | 163 of 163 (100.0%) |
| pg_catalog.pg_statio_all_indexes with the gate schema | 177 of 177 (100.0%) | 177 of 177 (100.0%) |
| pg_catalog.pg_statio_all_sequences | 0 of 0 | 0 of 0 |
| pg_catalog.pg_statio_all_sequences with the gate schema | 3 of 3 (100.0%) | 3 of 3 (100.0%) |
| pg_catalog.pg_statio_all_tables | 106 of 107 (99.1%) | 107 of 107 (100.0%) |
| pg_catalog.pg_statio_all_tables with the gate schema | 115 of 116 (99.1%) | 116 of 116 (100.0%) |
| pg_catalog.pg_statio_sys_indexes | 163 of 163 (100.0%) | 163 of 163 (100.0%) |
| pg_catalog.pg_statio_sys_indexes with the gate schema | 167 of 167 (100.0%) | 167 of 167 (100.0%) |
| pg_catalog.pg_statio_sys_sequences | 0 of 0 | 0 of 0 |
| pg_catalog.pg_statio_sys_sequences with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_statio_sys_tables | 106 of 107 (99.1%) | 107 of 107 (100.0%) |
| pg_catalog.pg_statio_sys_tables with the gate schema | 110 of 111 (99.1%) | 111 of 111 (100.0%) |
| pg_catalog.pg_statio_user_indexes | 0 of 0 | 0 of 0 |
| pg_catalog.pg_statio_user_indexes with the gate schema | 10 of 10 (100.0%) | 10 of 10 (100.0%) |
| pg_catalog.pg_statio_user_sequences | 0 of 0 | 0 of 0 |
| pg_catalog.pg_statio_user_sequences with the gate schema | 3 of 3 (100.0%) | 3 of 3 (100.0%) |
| pg_catalog.pg_statio_user_tables | 0 of 0 | 0 of 0 |
| pg_catalog.pg_statio_user_tables with the gate schema | 5 of 5 (100.0%) | 5 of 5 (100.0%) |
| pg_catalog.pg_statistic | 0 of 417 (0.0%) | 417 of 417 (100.0%) |
| pg_catalog.pg_statistic with the gate schema | 0 of 417 (0.0%) | 417 of 417 (100.0%) |
| pg_catalog.pg_statistic_ext | 0 of 0 | 0 of 0 |
| pg_catalog.pg_statistic_ext with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_statistic_ext_data | 0 of 0 | 0 of 0 |
| pg_catalog.pg_statistic_ext_data with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stats | 0 of 417 (0.0%) | 417 of 417 (100.0%) |
| pg_catalog.pg_stats with the gate schema | 0 of 417 (0.0%) | 417 of 417 (100.0%) |
| pg_catalog.pg_stats_ext | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stats_ext with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stats_ext_exprs | 0 of 0 | 0 of 0 |
| pg_catalog.pg_stats_ext_exprs with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_subscription | 0 of 0 | 0 of 0 |
| pg_catalog.pg_subscription with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_subscription_rel | 0 of 0 | 0 of 0 |
| pg_catalog.pg_subscription_rel with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_tables | 68 of 68 (100.0%) | 68 of 68 (100.0%) |
| pg_catalog.pg_tables with the gate schema | 73 of 73 (100.0%) | 73 of 73 (100.0%) |
| pg_catalog.pg_tablespace | 2 of 2 (100.0%) | 2 of 2 (100.0%) |
| pg_catalog.pg_tablespace with the gate schema | 2 of 2 (100.0%) | 2 of 2 (100.0%) |
| pg_catalog.pg_timezone_abbrevs | 195 of 195 (100.0%) | 195 of 195 (100.0%) |
| pg_catalog.pg_timezone_abbrevs with the gate schema | 195 of 195 (100.0%) | 195 of 195 (100.0%) |
| pg_catalog.pg_timezone_names | 598 of 598 (100.0%) | 598 of 598 (100.0%) |
| pg_catalog.pg_timezone_names with the gate schema | 598 of 598 (100.0%) | 598 of 598 (100.0%) |
| pg_catalog.pg_transform | 0 of 0 | 0 of 0 |
| pg_catalog.pg_transform with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_trigger | 0 of 0 | 0 of 0 |
| pg_catalog.pg_trigger with the gate schema | 12 of 12 (100.0%) | 12 of 12 (100.0%) |
| pg_catalog.pg_ts_config | 32 of 32 (100.0%) | 32 of 32 (100.0%) |
| pg_catalog.pg_ts_config with the gate schema | 32 of 32 (100.0%) | 32 of 32 (100.0%) |
| pg_catalog.pg_ts_config_map | 608 of 608 (100.0%) | 608 of 608 (100.0%) |
| pg_catalog.pg_ts_config_map with the gate schema | 608 of 608 (100.0%) | 608 of 608 (100.0%) |
| pg_catalog.pg_ts_dict | 32 of 32 (100.0%) | 32 of 32 (100.0%) |
| pg_catalog.pg_ts_dict with the gate schema | 32 of 32 (100.0%) | 32 of 32 (100.0%) |
| pg_catalog.pg_ts_parser | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_ts_parser with the gate schema | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_ts_template | 5 of 5 (100.0%) | 5 of 5 (100.0%) |
| pg_catalog.pg_ts_template with the gate schema | 5 of 5 (100.0%) | 5 of 5 (100.0%) |
| pg_catalog.pg_type | 637 of 637 (100.0%) | 637 of 637 (100.0%) |
| pg_catalog.pg_type with the gate schema | 649 of 649 (100.0%) | 649 of 649 (100.0%) |
| pg_catalog.pg_user | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_user with the gate schema | 1 of 1 (100.0%) | 1 of 1 (100.0%) |
| pg_catalog.pg_user_mapping | 0 of 0 | 0 of 0 |
| pg_catalog.pg_user_mapping with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_user_mappings | 0 of 0 | 0 of 0 |
| pg_catalog.pg_user_mappings with the gate schema | 0 of 0 | 0 of 0 |
| pg_catalog.pg_views | 151 of 151 (100.0%) | 151 of 151 (100.0%) |
| pg_catalog.pg_views with the gate schema | 152 of 152 (100.0%) | 152 of 152 (100.0%) |
| pg_catalog.pg_wait_events | 287 of 287 (100.0%) | 287 of 287 (100.0%) |
| pg_catalog.pg_wait_events with the gate schema | 287 of 287 (100.0%) | 287 of 287 (100.0%) |

### L3 Query

| Row | Unit | Server | Oracle |
|---|---|---|---|
| differential cases | cases | not run | not run |
| regression tests | tests | 63 of 239 (26.4%) | 239 of 239 (100.0%) |

### L4 Behave

| Row | Unit | Server | Oracle |
|---|---|---|---|
| Hermitage cases | cases | not run | not run |
| isolation specs | specs | 18 of 135 (13.3%) | 135 of 135 (100.0%) |
| parameter cases | cases | 1298 of 1303 (99.6%) | 1303 of 1303 (100.0%) |

### L5 Drop-in

| Row | Unit | Server | Oracle |
|---|---|---|---|
| client suites | tests | not run | not run |

### Exclusions

Each exclusion of spec/05 section 5.3 and each rule of section 5.2 that a run used, with the count of values that it applied to and the count of those that differed.

| Run | Side | Exclusion or rule | Applied | Differed |
|---|---|---|---|---|
| catalog | oracle | 2. values of user OIDs | 2103 | 0 |
| catalog | oracle | 3. values of statistics counters | 18304 | 0 |
| catalog | oracle | 4. physical row identity (ctid, xmin, xmax, cmin, cmax) | 857 | 0 |
| catalog | oracle | 5. sizes | 1505 | 0 |
| catalog | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 12 | 0 |
| catalog | oracle | 11. pg_node_tree and TOAST relations | 822 | 0 |
| catalog | oracle | 12. suffix of server_version | 4 | 0 |
| catalog | oracle | 13. process ID | 11 | 0 |
| catalog | oracle | 15. file system paths | 78 | 0 |
| catalog | oracle | 16. values of initdb OIDs that depend on the locales of the host | 8449 | 0 |
| catalog | oracle | rule: row order without ORDER BY (multiset) | 98 | 0 |
| catalog | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 0 |
| catalog | oracle | rule: port of the server | 3 | 0 |
| catalog | server | 2. values of user OIDs | 2115 | 702 |
| catalog | server | 3. values of statistics counters | 18429 | 12248 |
| catalog | server | 4. physical row identity (ctid, xmin, xmax, cmin, cmax) | 865 | 120 |
| catalog | server | 5. sizes | 1505 | 524 |
| catalog | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 12 | 12 |
| catalog | server | 11. pg_node_tree and TOAST relations | 822 | 216 |
| catalog | server | 12. suffix of server_version | 4 | 4 |
| catalog | server | 13. process ID | 10 | 10 |
| catalog | server | 15. file system paths | 78 | 78 |
| catalog | server | 16. values of initdb OIDs that depend on the locales of the host | 8449 | 8448 |
| catalog | server | rule: row order without ORDER BY (multiset) | 98 | 4 |
| catalog | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| catalog | server | rule: port of the server | 3 | 3 |
| catalog-gate | oracle | 2. values of user OIDs | 2986 | 0 |
| catalog-gate | oracle | 3. values of statistics counters | 18968 | 0 |
| catalog-gate | oracle | 4. physical row identity (ctid, xmin, xmax, cmin, cmax) | 917 | 0 |
| catalog-gate | oracle | 5. sizes | 1586 | 0 |
| catalog-gate | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 12 | 0 |
| catalog-gate | oracle | 11. pg_node_tree and TOAST relations | 908 | 0 |
| catalog-gate | oracle | 12. suffix of server_version | 4 | 0 |
| catalog-gate | oracle | 13. process ID | 10 | 0 |
| catalog-gate | oracle | 15. file system paths | 78 | 0 |
| catalog-gate | oracle | 16. values of initdb OIDs that depend on the locales of the host | 8449 | 0 |
| catalog-gate | oracle | rule: row order without ORDER BY (multiset) | 111 | 0 |
| catalog-gate | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 0 |
| catalog-gate | oracle | rule: port of the server | 3 | 0 |
| catalog-gate | server | 2. values of user OIDs | 2989 | 1576 |
| catalog-gate | server | 3. values of statistics counters | 19000 | 12474 |
| catalog-gate | server | 4. physical row identity (ctid, xmin, xmax, cmin, cmax) | 919 | 138 |
| catalog-gate | server | 5. sizes | 1586 | 524 |
| catalog-gate | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 12 | 12 |
| catalog-gate | server | 11. pg_node_tree and TOAST relations | 908 | 278 |
| catalog-gate | server | 12. suffix of server_version | 4 | 4 |
| catalog-gate | server | 13. process ID | 10 | 10 |
| catalog-gate | server | 15. file system paths | 78 | 78 |
| catalog-gate | server | 16. values of initdb OIDs that depend on the locales of the host | 8449 | 8448 |
| catalog-gate | server | rule: row order without ORDER BY (multiset) | 111 | 4 |
| catalog-gate | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| catalog-gate | server | rule: port of the server | 3 | 3 |
| parameters | oracle | 2. values of user OIDs | 17 | 0 |
| parameters | oracle | 5. sizes | 10 | 0 |
| parameters | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 1389 | 0 |
| parameters | oracle | 12. suffix of server_version | 5 | 0 |
| parameters | oracle | 15. file system paths | 41 | 0 |
| parameters | oracle | rule: row order without ORDER BY (multiset) | 2 | 0 |
| parameters | oracle | rule: port of the server | 4 | 0 |
| parameters | server | 2. values of user OIDs | 17 | 0 |
| parameters | server | 5. sizes | 10 | 4 |
| parameters | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 1389 | 1389 |
| parameters | server | 12. suffix of server_version | 5 | 5 |
| parameters | server | 15. file system paths | 41 | 41 |
| parameters | server | rule: row order without ORDER BY (multiset) | 2 | 0 |
| parameters | server | rule: port of the server | 4 | 4 |
| replay-activerecord | oracle | 2. values of user OIDs | 318 | 70 |
| replay-activerecord | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-activerecord | oracle | 12. suffix of server_version | 1 | 0 |
| replay-activerecord | oracle | 13. process ID | 1 | 1 |
| replay-activerecord | oracle | 16. values of initdb OIDs that depend on the locales of the host | 60 | 60 |
| replay-activerecord | oracle | rule: row order without ORDER BY (multiset) | 14 | 0 |
| replay-activerecord | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-activerecord | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-activerecord | server | 2. values of user OIDs | 318 | 70 |
| replay-activerecord | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-activerecord | server | 12. suffix of server_version | 1 | 1 |
| replay-activerecord | server | 13. process ID | 1 | 1 |
| replay-activerecord | server | 16. values of initdb OIDs that depend on the locales of the host | 60 | 60 |
| replay-activerecord | server | rule: row order without ORDER BY (multiset) | 14 | 0 |
| replay-activerecord | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-activerecord | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-asyncpg | oracle | 2. values of user OIDs | 6 | 3 |
| replay-asyncpg | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-asyncpg | oracle | 12. suffix of server_version | 1 | 0 |
| replay-asyncpg | oracle | 13. process ID | 1 | 1 |
| replay-asyncpg | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-asyncpg | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-asyncpg | server | 2. values of user OIDs | 6 | 3 |
| replay-asyncpg | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-asyncpg | server | 12. suffix of server_version | 1 | 1 |
| replay-asyncpg | server | 13. process ID | 1 | 1 |
| replay-asyncpg | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-asyncpg | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-diesel | oracle | 2. values of user OIDs | 40 | 5 |
| replay-diesel | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 12 | 4 |
| replay-diesel | oracle | 12. suffix of server_version | 7 | 0 |
| replay-diesel | oracle | 13. process ID | 7 | 7 |
| replay-diesel | oracle | 16. values of initdb OIDs that depend on the locales of the host | 18 | 18 |
| replay-diesel | oracle | rule: row order without ORDER BY (multiset) | 1 | 0 |
| replay-diesel | oracle | rule: BackendKeyData secret key (length only) | 7 | 7 |
| replay-diesel | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 14 | 14 |
| replay-diesel | server | 2. values of user OIDs | 40 | 5 |
| replay-diesel | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 12 | 12 |
| replay-diesel | server | 12. suffix of server_version | 7 | 7 |
| replay-diesel | server | 13. process ID | 7 | 7 |
| replay-diesel | server | 16. values of initdb OIDs that depend on the locales of the host | 18 | 18 |
| replay-diesel | server | rule: row order without ORDER BY (multiset) | 1 | 0 |
| replay-diesel | server | rule: BackendKeyData secret key (length only) | 7 | 7 |
| replay-diesel | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 14 | 14 |
| replay-django | oracle | 2. values of user OIDs | 54 | 21 |
| replay-django | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 3 | 1 |
| replay-django | oracle | 12. suffix of server_version | 1 | 0 |
| replay-django | oracle | 13. process ID | 1 | 1 |
| replay-django | oracle | rule: row order without ORDER BY (multiset) | 1 | 0 |
| replay-django | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-django | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-django | server | 2. values of user OIDs | 54 | 21 |
| replay-django | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 3 | 3 |
| replay-django | server | 12. suffix of server_version | 1 | 1 |
| replay-django | server | 13. process ID | 1 | 1 |
| replay-django | server | rule: row order without ORDER BY (multiset) | 1 | 0 |
| replay-django | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-django | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-drizzle | oracle | 2. values of user OIDs | 318 | 42 |
| replay-drizzle | oracle | 3. values of statistics counters | 8 | 0 |
| replay-drizzle | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 3 | 1 |
| replay-drizzle | oracle | 12. suffix of server_version | 5 | 0 |
| replay-drizzle | oracle | 13. process ID | 5 | 5 |
| replay-drizzle | oracle | 16. values of initdb OIDs that depend on the locales of the host | 88 | 88 |
| replay-drizzle | oracle | rule: row order without ORDER BY (multiset) | 20 | 0 |
| replay-drizzle | oracle | rule: BackendKeyData secret key (length only) | 5 | 5 |
| replay-drizzle | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 10 | 10 |
| replay-drizzle | server | 2. values of user OIDs | 318 | 42 |
| replay-drizzle | server | 3. values of statistics counters | 8 | 0 |
| replay-drizzle | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 3 | 3 |
| replay-drizzle | server | 12. suffix of server_version | 5 | 5 |
| replay-drizzle | server | 13. process ID | 5 | 5 |
| replay-drizzle | server | 16. values of initdb OIDs that depend on the locales of the host | 88 | 88 |
| replay-drizzle | server | rule: row order without ORDER BY (multiset) | 20 | 0 |
| replay-drizzle | server | rule: BackendKeyData secret key (length only) | 5 | 5 |
| replay-drizzle | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 10 | 10 |
| replay-ecto | oracle | 2. values of user OIDs | 115 | 55 |
| replay-ecto | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 9 | 3 |
| replay-ecto | oracle | 12. suffix of server_version | 2 | 0 |
| replay-ecto | oracle | 13. process ID | 2 | 2 |
| replay-ecto | oracle | 16. values of initdb OIDs that depend on the locales of the host | 15 | 15 |
| replay-ecto | oracle | rule: row order without ORDER BY (multiset) | 3 | 0 |
| replay-ecto | oracle | rule: BackendKeyData secret key (length only) | 2 | 2 |
| replay-ecto | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 4 | 4 |
| replay-ecto | server | 2. values of user OIDs | 115 | 55 |
| replay-ecto | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 9 | 9 |
| replay-ecto | server | 12. suffix of server_version | 2 | 2 |
| replay-ecto | server | 13. process ID | 2 | 2 |
| replay-ecto | server | 16. values of initdb OIDs that depend on the locales of the host | 15 | 15 |
| replay-ecto | server | rule: row order without ORDER BY (multiset) | 3 | 0 |
| replay-ecto | server | rule: BackendKeyData secret key (length only) | 2 | 2 |
| replay-ecto | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 4 | 4 |
| replay-efcore | oracle | 2. values of user OIDs | 305 | 61 |
| replay-efcore | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 17 | 11 |
| replay-efcore | oracle | 12. suffix of server_version | 29 | 0 |
| replay-efcore | oracle | 13. process ID | 21 | 21 |
| replay-efcore | oracle | 16. values of initdb OIDs that depend on the locales of the host | 1860 | 1860 |
| replay-efcore | oracle | rule: row order without ORDER BY (multiset) | 17 | 0 |
| replay-efcore | oracle | rule: BackendKeyData secret key (length only) | 21 | 21 |
| replay-efcore | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 42 | 42 |
| replay-efcore | server | 2. values of user OIDs | 305 | 61 |
| replay-efcore | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 17 | 17 |
| replay-efcore | server | 12. suffix of server_version | 29 | 29 |
| replay-efcore | server | 13. process ID | 21 | 21 |
| replay-efcore | server | 16. values of initdb OIDs that depend on the locales of the host | 1794 | 1794 |
| replay-efcore | server | rule: row order without ORDER BY (multiset) | 17 | 0 |
| replay-efcore | server | rule: BackendKeyData secret key (length only) | 21 | 21 |
| replay-efcore | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 42 | 42 |
| replay-flyway | oracle | 2. values of user OIDs | 144 | 80 |
| replay-flyway | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 10 | 8 |
| replay-flyway | oracle | 12. suffix of server_version | 16 | 0 |
| replay-flyway | oracle | 13. process ID | 9 | 9 |
| replay-flyway | oracle | 16. values of initdb OIDs that depend on the locales of the host | 4 | 4 |
| replay-flyway | oracle | rule: row order without ORDER BY (multiset) | 1 | 0 |
| replay-flyway | oracle | rule: BackendKeyData secret key (length only) | 9 | 9 |
| replay-flyway | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 18 | 18 |
| replay-flyway | server | 2. values of user OIDs | 144 | 80 |
| replay-flyway | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 10 | 10 |
| replay-flyway | server | 12. suffix of server_version | 16 | 16 |
| replay-flyway | server | 13. process ID | 9 | 9 |
| replay-flyway | server | 16. values of initdb OIDs that depend on the locales of the host | 4 | 4 |
| replay-flyway | server | rule: row order without ORDER BY (multiset) | 1 | 0 |
| replay-flyway | server | rule: BackendKeyData secret key (length only) | 9 | 9 |
| replay-flyway | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 18 | 18 |
| replay-golang-migrate | oracle | 2. values of user OIDs | 4 | 2 |
| replay-golang-migrate | oracle | 12. suffix of server_version | 1 | 0 |
| replay-golang-migrate | oracle | 13. process ID | 1 | 1 |
| replay-golang-migrate | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-golang-migrate | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-golang-migrate | server | 2. values of user OIDs | 4 | 2 |
| replay-golang-migrate | server | 12. suffix of server_version | 1 | 1 |
| replay-golang-migrate | server | 13. process ID | 1 | 1 |
| replay-golang-migrate | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-golang-migrate | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-grafana | oracle | 2. values of user OIDs | 10 | 4 |
| replay-grafana | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 3 | 1 |
| replay-grafana | oracle | 12. suffix of server_version | 3 | 0 |
| replay-grafana | oracle | 13. process ID | 3 | 3 |
| replay-grafana | oracle | 16. values of initdb OIDs that depend on the locales of the host | 1 | 1 |
| replay-grafana | oracle | rule: row order without ORDER BY (multiset) | 1 | 0 |
| replay-grafana | oracle | rule: BackendKeyData secret key (length only) | 3 | 3 |
| replay-grafana | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 6 | 6 |
| replay-grafana | server | 2. values of user OIDs | 10 | 4 |
| replay-grafana | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 3 | 3 |
| replay-grafana | server | 12. suffix of server_version | 3 | 3 |
| replay-grafana | server | 13. process ID | 3 | 3 |
| replay-grafana | server | 16. values of initdb OIDs that depend on the locales of the host | 1 | 1 |
| replay-grafana | server | rule: row order without ORDER BY (multiset) | 1 | 0 |
| replay-grafana | server | rule: BackendKeyData secret key (length only) | 3 | 3 |
| replay-grafana | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 6 | 6 |
| replay-hibernate | oracle | 2. values of user OIDs | 85 | 29 |
| replay-hibernate | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 5 | 3 |
| replay-hibernate | oracle | 12. suffix of server_version | 6 | 0 |
| replay-hibernate | oracle | 13. process ID | 4 | 4 |
| replay-hibernate | oracle | 16. values of initdb OIDs that depend on the locales of the host | 14 | 14 |
| replay-hibernate | oracle | rule: row order without ORDER BY (multiset) | 1 | 0 |
| replay-hibernate | oracle | rule: BackendKeyData secret key (length only) | 4 | 4 |
| replay-hibernate | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 8 | 8 |
| replay-hibernate | server | 2. values of user OIDs | 85 | 29 |
| replay-hibernate | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 5 | 5 |
| replay-hibernate | server | 12. suffix of server_version | 6 | 6 |
| replay-hibernate | server | 13. process ID | 4 | 4 |
| replay-hibernate | server | 16. values of initdb OIDs that depend on the locales of the host | 14 | 14 |
| replay-hibernate | server | rule: row order without ORDER BY (multiset) | 1 | 0 |
| replay-hibernate | server | rule: BackendKeyData secret key (length only) | 4 | 4 |
| replay-hibernate | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 8 | 8 |
| replay-lib-pq | oracle | 2. values of user OIDs | 36 | 18 |
| replay-lib-pq | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-lib-pq | oracle | 12. suffix of server_version | 1 | 0 |
| replay-lib-pq | oracle | 13. process ID | 1 | 1 |
| replay-lib-pq | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-lib-pq | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-lib-pq | server | 2. values of user OIDs | 36 | 18 |
| replay-lib-pq | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-lib-pq | server | 12. suffix of server_version | 1 | 1 |
| replay-lib-pq | server | 13. process ID | 1 | 1 |
| replay-lib-pq | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-lib-pq | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-libpq | oracle | 2. values of user OIDs | 36 | 18 |
| replay-libpq | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-libpq | oracle | 12. suffix of server_version | 1 | 0 |
| replay-libpq | oracle | 13. process ID | 1 | 1 |
| replay-libpq | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-libpq | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-libpq | server | 2. values of user OIDs | 36 | 18 |
| replay-libpq | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-libpq | server | 12. suffix of server_version | 1 | 1 |
| replay-libpq | server | 13. process ID | 1 | 1 |
| replay-libpq | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-libpq | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-liquibase | oracle | 2. values of user OIDs | 562 | 89 |
| replay-liquibase | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 21 | 15 |
| replay-liquibase | oracle | 12. suffix of server_version | 18 | 0 |
| replay-liquibase | oracle | 13. process ID | 6 | 6 |
| replay-liquibase | oracle | 16. values of initdb OIDs that depend on the locales of the host | 25 | 25 |
| replay-liquibase | oracle | rule: row order without ORDER BY (multiset) | 7 | 0 |
| replay-liquibase | oracle | rule: BackendKeyData secret key (length only) | 6 | 6 |
| replay-liquibase | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 12 | 12 |
| replay-liquibase | server | 2. values of user OIDs | 562 | 89 |
| replay-liquibase | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 21 | 21 |
| replay-liquibase | server | 12. suffix of server_version | 18 | 18 |
| replay-liquibase | server | 13. process ID | 6 | 6 |
| replay-liquibase | server | 16. values of initdb OIDs that depend on the locales of the host | 25 | 25 |
| replay-liquibase | server | rule: row order without ORDER BY (multiset) | 7 | 0 |
| replay-liquibase | server | rule: BackendKeyData secret key (length only) | 6 | 6 |
| replay-liquibase | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 12 | 12 |
| replay-metabase | oracle | 2. values of user OIDs | 126 | 35 |
| replay-metabase | oracle | 3. values of statistics counters | 2 | 0 |
| replay-metabase | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 3 | 1 |
| replay-metabase | oracle | 12. suffix of server_version | 5 | 0 |
| replay-metabase | oracle | 13. process ID | 5 | 5 |
| replay-metabase | oracle | rule: row order without ORDER BY (multiset) | 7 | 0 |
| replay-metabase | oracle | rule: BackendKeyData secret key (length only) | 5 | 5 |
| replay-metabase | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 10 | 10 |
| replay-metabase | server | 2. values of user OIDs | 126 | 35 |
| replay-metabase | server | 3. values of statistics counters | 2 | 0 |
| replay-metabase | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 3 | 3 |
| replay-metabase | server | 12. suffix of server_version | 5 | 5 |
| replay-metabase | server | 13. process ID | 5 | 5 |
| replay-metabase | server | rule: row order without ORDER BY (multiset) | 7 | 0 |
| replay-metabase | server | rule: BackendKeyData secret key (length only) | 5 | 5 |
| replay-metabase | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 10 | 10 |
| replay-node-postgres | oracle | 2. values of user OIDs | 36 | 18 |
| replay-node-postgres | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-node-postgres | oracle | 12. suffix of server_version | 1 | 0 |
| replay-node-postgres | oracle | 13. process ID | 1 | 1 |
| replay-node-postgres | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-node-postgres | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-node-postgres | server | 2. values of user OIDs | 36 | 18 |
| replay-node-postgres | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-node-postgres | server | 12. suffix of server_version | 1 | 1 |
| replay-node-postgres | server | 13. process ID | 1 | 1 |
| replay-node-postgres | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-node-postgres | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-npgsql | oracle | 2. values of user OIDs | 16 | 3 |
| replay-npgsql | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 7 | 3 |
| replay-npgsql | oracle | 12. suffix of server_version | 2 | 0 |
| replay-npgsql | oracle | 13. process ID | 1 | 1 |
| replay-npgsql | oracle | 16. values of initdb OIDs that depend on the locales of the host | 15 | 15 |
| replay-npgsql | oracle | rule: row order without ORDER BY (multiset) | 1 | 0 |
| replay-npgsql | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-npgsql | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-npgsql | server | 2. values of user OIDs | 16 | 3 |
| replay-npgsql | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 7 | 7 |
| replay-npgsql | server | 12. suffix of server_version | 2 | 2 |
| replay-npgsql | server | 13. process ID | 1 | 1 |
| replay-npgsql | server | 16. values of initdb OIDs that depend on the locales of the host | 15 | 15 |
| replay-npgsql | server | rule: row order without ORDER BY (multiset) | 1 | 0 |
| replay-npgsql | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-npgsql | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-pg_dump | oracle | 2. values of user OIDs | 1343 | 328 |
| replay-pg_dump | oracle | 3. values of statistics counters | 452 | 8 |
| replay-pg_dump | oracle | 4. physical row identity (ctid, xmin, xmax, cmin, cmax) | 1054 | 24 |
| replay-pg_dump | oracle | 5. sizes | 1436 | 40 |
| replay-pg_dump | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 3 | 1 |
| replay-pg_dump | oracle | 12. suffix of server_version | 5 | 0 |
| replay-pg_dump | oracle | 13. process ID | 5 | 5 |
| replay-pg_dump | oracle | 16. values of initdb OIDs that depend on the locales of the host | 13056 | 13056 |
| replay-pg_dump | oracle | rule: row order without ORDER BY (multiset) | 360 | 0 |
| replay-pg_dump | oracle | rule: CopyData boundaries (joined) | 8 | 0 |
| replay-pg_dump | oracle | rule: BackendKeyData secret key (length only) | 5 | 5 |
| replay-pg_dump | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 10 | 10 |
| replay-pg_dump | server | 2. values of user OIDs | 1343 | 330 |
| replay-pg_dump | server | 3. values of statistics counters | 452 | 141 |
| replay-pg_dump | server | 4. physical row identity (ctid, xmin, xmax, cmin, cmax) | 1054 | 233 |
| replay-pg_dump | server | 5. sizes | 1436 | 247 |
| replay-pg_dump | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 3 | 3 |
| replay-pg_dump | server | 12. suffix of server_version | 5 | 5 |
| replay-pg_dump | server | 13. process ID | 5 | 5 |
| replay-pg_dump | server | 16. values of initdb OIDs that depend on the locales of the host | 13638 | 13638 |
| replay-pg_dump | server | rule: row order without ORDER BY (multiset) | 360 | 0 |
| replay-pg_dump | server | rule: CopyData boundaries (joined) | 8 | 0 |
| replay-pg_dump | server | rule: BackendKeyData secret key (length only) | 5 | 5 |
| replay-pg_dump | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 10 | 10 |
| replay-pgadmin4 | oracle | 2. values of user OIDs | 380 | 89 |
| replay-pgadmin4 | oracle | 3. values of statistics counters | 2 | 0 |
| replay-pgadmin4 | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 5 | 3 |
| replay-pgadmin4 | oracle | 12. suffix of server_version | 6 | 0 |
| replay-pgadmin4 | oracle | 13. process ID | 4 | 4 |
| replay-pgadmin4 | oracle | 16. values of initdb OIDs that depend on the locales of the host | 4 | 4 |
| replay-pgadmin4 | oracle | rule: row order without ORDER BY (multiset) | 3 | 0 |
| replay-pgadmin4 | oracle | rule: BackendKeyData secret key (length only) | 4 | 4 |
| replay-pgadmin4 | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 8 | 8 |
| replay-pgadmin4 | server | 2. values of user OIDs | 380 | 89 |
| replay-pgadmin4 | server | 3. values of statistics counters | 2 | 0 |
| replay-pgadmin4 | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 5 | 5 |
| replay-pgadmin4 | server | 12. suffix of server_version | 6 | 6 |
| replay-pgadmin4 | server | 13. process ID | 4 | 4 |
| replay-pgadmin4 | server | 16. values of initdb OIDs that depend on the locales of the host | 4 | 4 |
| replay-pgadmin4 | server | rule: row order without ORDER BY (multiset) | 3 | 0 |
| replay-pgadmin4 | server | rule: BackendKeyData secret key (length only) | 4 | 4 |
| replay-pgadmin4 | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 8 | 8 |
| replay-pgbouncer | oracle | 2. values of user OIDs | 117 | 58 |
| replay-pgbouncer | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 18 | 2 |
| replay-pgbouncer | oracle | 12. suffix of server_version | 2 | 0 |
| replay-pgbouncer | oracle | 13. process ID | 2 | 2 |
| replay-pgbouncer | oracle | rule: BackendKeyData secret key (length only) | 2 | 2 |
| replay-pgbouncer | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 4 | 4 |
| replay-pgbouncer | server | 2. values of user OIDs | 117 | 58 |
| replay-pgbouncer | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 18 | 18 |
| replay-pgbouncer | server | 12. suffix of server_version | 2 | 2 |
| replay-pgbouncer | server | 13. process ID | 2 | 2 |
| replay-pgbouncer | server | rule: BackendKeyData secret key (length only) | 2 | 2 |
| replay-pgbouncer | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 4 | 4 |
| replay-pgcat | oracle | 2. values of user OIDs | 117 | 58 |
| replay-pgcat | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 18 | 2 |
| replay-pgcat | oracle | 12. suffix of server_version | 2 | 0 |
| replay-pgcat | oracle | 13. process ID | 2 | 2 |
| replay-pgcat | oracle | rule: BackendKeyData secret key (length only) | 2 | 2 |
| replay-pgcat | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 4 | 4 |
| replay-pgcat | server | 2. values of user OIDs | 117 | 58 |
| replay-pgcat | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 18 | 18 |
| replay-pgcat | server | 12. suffix of server_version | 2 | 2 |
| replay-pgcat | server | 13. process ID | 2 | 2 |
| replay-pgcat | server | rule: BackendKeyData secret key (length only) | 2 | 2 |
| replay-pgcat | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 4 | 4 |
| replay-pgjdbc | oracle | 2. values of user OIDs | 30 | 15 |
| replay-pgjdbc | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-pgjdbc | oracle | 12. suffix of server_version | 1 | 0 |
| replay-pgjdbc | oracle | 13. process ID | 1 | 1 |
| replay-pgjdbc | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-pgjdbc | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-pgjdbc | server | 2. values of user OIDs | 30 | 15 |
| replay-pgjdbc | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-pgjdbc | server | 12. suffix of server_version | 1 | 1 |
| replay-pgjdbc | server | 13. process ID | 1 | 1 |
| replay-pgjdbc | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-pgjdbc | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-pgx | oracle | 2. values of user OIDs | 6 | 3 |
| replay-pgx | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-pgx | oracle | 12. suffix of server_version | 1 | 0 |
| replay-pgx | oracle | 13. process ID | 1 | 1 |
| replay-pgx | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-pgx | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-pgx | server | 2. values of user OIDs | 6 | 3 |
| replay-pgx | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-pgx | server | 12. suffix of server_version | 1 | 1 |
| replay-pgx | server | 13. process ID | 1 | 1 |
| replay-pgx | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-pgx | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-postgres-js | oracle | 2. values of user OIDs | 8 | 3 |
| replay-postgres-js | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-postgres-js | oracle | 12. suffix of server_version | 1 | 0 |
| replay-postgres-js | oracle | 13. process ID | 1 | 1 |
| replay-postgres-js | oracle | 16. values of initdb OIDs that depend on the locales of the host | 282 | 282 |
| replay-postgres-js | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-postgres-js | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-postgres-js | server | 2. values of user OIDs | 8 | 3 |
| replay-postgres-js | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-postgres-js | server | 12. suffix of server_version | 1 | 1 |
| replay-postgres-js | server | 13. process ID | 1 | 1 |
| replay-postgres-js | server | 16. values of initdb OIDs that depend on the locales of the host | 282 | 282 |
| replay-postgres-js | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-postgres-js | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-postgrex | oracle | 2. values of user OIDs | 12 | 3 |
| replay-postgrex | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-postgrex | oracle | 12. suffix of server_version | 1 | 0 |
| replay-postgrex | oracle | 13. process ID | 1 | 1 |
| replay-postgrex | oracle | 16. values of initdb OIDs that depend on the locales of the host | 15 | 15 |
| replay-postgrex | oracle | rule: row order without ORDER BY (multiset) | 1 | 0 |
| replay-postgrex | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-postgrex | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-postgrex | server | 2. values of user OIDs | 12 | 3 |
| replay-postgrex | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-postgrex | server | 12. suffix of server_version | 1 | 1 |
| replay-postgrex | server | 13. process ID | 1 | 1 |
| replay-postgrex | server | 16. values of initdb OIDs that depend on the locales of the host | 15 | 15 |
| replay-postgrex | server | rule: row order without ORDER BY (multiset) | 1 | 0 |
| replay-postgrex | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-postgrex | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-prisma | oracle | 2. values of user OIDs | 212 | 74 |
| replay-prisma | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 15 | 11 |
| replay-prisma | oracle | 12. suffix of server_version | 16 | 0 |
| replay-prisma | oracle | 13. process ID | 7 | 7 |
| replay-prisma | oracle | 16. values of initdb OIDs that depend on the locales of the host | 14 | 14 |
| replay-prisma | oracle | rule: row order without ORDER BY (multiset) | 5 | 0 |
| replay-prisma | oracle | rule: BackendKeyData secret key (length only) | 7 | 7 |
| replay-prisma | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 14 | 14 |
| replay-prisma | server | 2. values of user OIDs | 212 | 74 |
| replay-prisma | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 15 | 15 |
| replay-prisma | server | 12. suffix of server_version | 16 | 16 |
| replay-prisma | server | 13. process ID | 7 | 7 |
| replay-prisma | server | 16. values of initdb OIDs that depend on the locales of the host | 14 | 14 |
| replay-prisma | server | rule: row order without ORDER BY (multiset) | 5 | 0 |
| replay-prisma | server | rule: BackendKeyData secret key (length only) | 7 | 7 |
| replay-prisma | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 14 | 14 |
| replay-psql | oracle | 2. values of user OIDs | 36 | 18 |
| replay-psql | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-psql | oracle | 12. suffix of server_version | 1 | 0 |
| replay-psql | oracle | 13. process ID | 1 | 1 |
| replay-psql | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-psql | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-psql | server | 2. values of user OIDs | 36 | 18 |
| replay-psql | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-psql | server | 12. suffix of server_version | 1 | 1 |
| replay-psql | server | 13. process ID | 1 | 1 |
| replay-psql | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-psql | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-psqlodbc | oracle | 2. values of user OIDs | 44 | 21 |
| replay-psqlodbc | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-psqlodbc | oracle | 12. suffix of server_version | 1 | 0 |
| replay-psqlodbc | oracle | 13. process ID | 1 | 1 |
| replay-psqlodbc | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-psqlodbc | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-psqlodbc | server | 2. values of user OIDs | 44 | 21 |
| replay-psqlodbc | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-psqlodbc | server | 12. suffix of server_version | 1 | 1 |
| replay-psqlodbc | server | 13. process ID | 1 | 1 |
| replay-psqlodbc | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-psqlodbc | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-psycopg | oracle | 2. values of user OIDs | 36 | 18 |
| replay-psycopg | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-psycopg | oracle | 12. suffix of server_version | 1 | 0 |
| replay-psycopg | oracle | 13. process ID | 1 | 1 |
| replay-psycopg | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-psycopg | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-psycopg | server | 2. values of user OIDs | 36 | 18 |
| replay-psycopg | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-psycopg | server | 12. suffix of server_version | 1 | 1 |
| replay-psycopg | server | 13. process ID | 1 | 1 |
| replay-psycopg | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-psycopg | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-psycopg2 | oracle | 2. values of user OIDs | 36 | 18 |
| replay-psycopg2 | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-psycopg2 | oracle | 12. suffix of server_version | 1 | 0 |
| replay-psycopg2 | oracle | 13. process ID | 1 | 1 |
| replay-psycopg2 | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-psycopg2 | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-psycopg2 | server | 2. values of user OIDs | 36 | 18 |
| replay-psycopg2 | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-psycopg2 | server | 12. suffix of server_version | 1 | 1 |
| replay-psycopg2 | server | 13. process ID | 1 | 1 |
| replay-psycopg2 | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-psycopg2 | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-r2dbc-postgresql | oracle | 2. values of user OIDs | 69 | 18 |
| replay-r2dbc-postgresql | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-r2dbc-postgresql | oracle | 12. suffix of server_version | 1 | 0 |
| replay-r2dbc-postgresql | oracle | 13. process ID | 1 | 1 |
| replay-r2dbc-postgresql | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-r2dbc-postgresql | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-r2dbc-postgresql | server | 2. values of user OIDs | 69 | 18 |
| replay-r2dbc-postgresql | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-r2dbc-postgresql | server | 12. suffix of server_version | 1 | 1 |
| replay-r2dbc-postgresql | server | 13. process ID | 1 | 1 |
| replay-r2dbc-postgresql | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-r2dbc-postgresql | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-ruby-pg | oracle | 2. values of user OIDs | 36 | 18 |
| replay-ruby-pg | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-ruby-pg | oracle | 12. suffix of server_version | 1 | 0 |
| replay-ruby-pg | oracle | 13. process ID | 1 | 1 |
| replay-ruby-pg | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-ruby-pg | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-ruby-pg | server | 2. values of user OIDs | 36 | 18 |
| replay-ruby-pg | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-ruby-pg | server | 12. suffix of server_version | 1 | 1 |
| replay-ruby-pg | server | 13. process ID | 1 | 1 |
| replay-ruby-pg | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-ruby-pg | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-sequelize | oracle | 2. values of user OIDs | 151 | 43 |
| replay-sequelize | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 3 | 1 |
| replay-sequelize | oracle | 12. suffix of server_version | 2 | 0 |
| replay-sequelize | oracle | 13. process ID | 2 | 2 |
| replay-sequelize | oracle | 16. values of initdb OIDs that depend on the locales of the host | 187 | 187 |
| replay-sequelize | oracle | rule: row order without ORDER BY (multiset) | 4 | 0 |
| replay-sequelize | oracle | rule: BackendKeyData secret key (length only) | 2 | 2 |
| replay-sequelize | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 4 | 4 |
| replay-sequelize | server | 2. values of user OIDs | 151 | 43 |
| replay-sequelize | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 3 | 3 |
| replay-sequelize | server | 12. suffix of server_version | 2 | 2 |
| replay-sequelize | server | 13. process ID | 2 | 2 |
| replay-sequelize | server | 16. values of initdb OIDs that depend on the locales of the host | 187 | 187 |
| replay-sequelize | server | rule: row order without ORDER BY (multiset) | 4 | 0 |
| replay-sequelize | server | rule: BackendKeyData secret key (length only) | 2 | 2 |
| replay-sequelize | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 4 | 4 |
| replay-sqlalchemy | oracle | 2. values of user OIDs | 121 | 28 |
| replay-sqlalchemy | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 4 | 2 |
| replay-sqlalchemy | oracle | 12. suffix of server_version | 2 | 0 |
| replay-sqlalchemy | oracle | 13. process ID | 1 | 1 |
| replay-sqlalchemy | oracle | rule: row order without ORDER BY (multiset) | 2 | 0 |
| replay-sqlalchemy | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-sqlalchemy | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-sqlalchemy | server | 2. values of user OIDs | 121 | 28 |
| replay-sqlalchemy | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 4 | 4 |
| replay-sqlalchemy | server | 12. suffix of server_version | 2 | 2 |
| replay-sqlalchemy | server | 13. process ID | 1 | 1 |
| replay-sqlalchemy | server | rule: row order without ORDER BY (multiset) | 2 | 0 |
| replay-sqlalchemy | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-sqlalchemy | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-sqlx | oracle | 2. values of user OIDs | 6 | 3 |
| replay-sqlx | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-sqlx | oracle | 12. suffix of server_version | 1 | 0 |
| replay-sqlx | oracle | 13. process ID | 1 | 1 |
| replay-sqlx | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-sqlx | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-sqlx | server | 2. values of user OIDs | 6 | 3 |
| replay-sqlx | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-sqlx | server | 12. suffix of server_version | 1 | 1 |
| replay-sqlx | server | 13. process ID | 1 | 1 |
| replay-sqlx | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-sqlx | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-supavisor | oracle | 2. values of user OIDs | 123 | 58 |
| replay-supavisor | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 19 | 3 |
| replay-supavisor | oracle | 12. suffix of server_version | 5 | 0 |
| replay-supavisor | oracle | 13. process ID | 4 | 4 |
| replay-supavisor | oracle | 16. values of initdb OIDs that depend on the locales of the host | 15 | 15 |
| replay-supavisor | oracle | rule: row order without ORDER BY (multiset) | 1 | 0 |
| replay-supavisor | oracle | rule: BackendKeyData secret key (length only) | 4 | 4 |
| replay-supavisor | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 8 | 8 |
| replay-supavisor | server | 2. values of user OIDs | 123 | 58 |
| replay-supavisor | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 19 | 19 |
| replay-supavisor | server | 12. suffix of server_version | 5 | 5 |
| replay-supavisor | server | 13. process ID | 4 | 4 |
| replay-supavisor | server | 16. values of initdb OIDs that depend on the locales of the host | 15 | 15 |
| replay-supavisor | server | rule: row order without ORDER BY (multiset) | 1 | 0 |
| replay-supavisor | server | rule: BackendKeyData secret key (length only) | 4 | 4 |
| replay-supavisor | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 8 | 8 |
| replay-superset | oracle | 2. values of user OIDs | 228 | 53 |
| replay-superset | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 22 | 20 |
| replay-superset | oracle | 12. suffix of server_version | 53 | 0 |
| replay-superset | oracle | 13. process ID | 36 | 36 |
| replay-superset | oracle | rule: row order without ORDER BY (multiset) | 5 | 0 |
| replay-superset | oracle | rule: BackendKeyData secret key (length only) | 34 | 34 |
| replay-superset | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 68 | 68 |
| replay-superset | server | 2. values of user OIDs | 228 | 53 |
| replay-superset | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 22 | 22 |
| replay-superset | server | 12. suffix of server_version | 53 | 53 |
| replay-superset | server | 13. process ID | 36 | 36 |
| replay-superset | server | rule: row order without ORDER BY (multiset) | 5 | 0 |
| replay-superset | server | rule: BackendKeyData secret key (length only) | 34 | 34 |
| replay-superset | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 68 | 68 |
| replay-tokio-postgres | oracle | 2. values of user OIDs | 36 | 18 |
| replay-tokio-postgres | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 2 |
| replay-tokio-postgres | oracle | 12. suffix of server_version | 1 | 0 |
| replay-tokio-postgres | oracle | 13. process ID | 1 | 1 |
| replay-tokio-postgres | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-tokio-postgres | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-tokio-postgres | server | 2. values of user OIDs | 36 | 18 |
| replay-tokio-postgres | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 6 | 6 |
| replay-tokio-postgres | server | 12. suffix of server_version | 1 | 1 |
| replay-tokio-postgres | server | 13. process ID | 1 | 1 |
| replay-tokio-postgres | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-tokio-postgres | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-typeorm | oracle | 2. values of user OIDs | 190 | 18 |
| replay-typeorm | oracle | 10. platform text of version() and pg_config, and the F, L, R error fields | 4 | 2 |
| replay-typeorm | oracle | 12. suffix of server_version | 2 | 0 |
| replay-typeorm | oracle | 13. process ID | 1 | 1 |
| replay-typeorm | oracle | 16. values of initdb OIDs that depend on the locales of the host | 118 | 118 |
| replay-typeorm | oracle | rule: row order without ORDER BY (multiset) | 7 | 0 |
| replay-typeorm | oracle | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-typeorm | oracle | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |
| replay-typeorm | server | 2. values of user OIDs | 190 | 18 |
| replay-typeorm | server | 10. platform text of version() and pg_config, and the F, L, R error fields | 4 | 4 |
| replay-typeorm | server | 12. suffix of server_version | 2 | 2 |
| replay-typeorm | server | 13. process ID | 1 | 1 |
| replay-typeorm | server | 16. values of initdb OIDs that depend on the locales of the host | 118 | 118 |
| replay-typeorm | server | rule: row order without ORDER BY (multiset) | 7 | 0 |
| replay-typeorm | server | rule: BackendKeyData secret key (length only) | 1 | 1 |
| replay-typeorm | server | rule: SCRAM nonces, salt, proof, signature and stored keys | 2 | 2 |

## PostgreSQL 18

PostgreSQL 18 is a shim of `rupg.compat_version`.

No run has a result file for this version.

## PostgreSQL 17

PostgreSQL 17 is a shim of `rupg.compat_version`.

No run has a result file for this version.

## PostgreSQL 16

PostgreSQL 16 is a shim of `rupg.compat_version`.

No run has a result file for this version.

## PostgreSQL 15

PostgreSQL 15 is a shim of `rupg.compat_version`.

No run has a result file for this version.

## PostgreSQL 14

PostgreSQL 14 is a shim of `rupg.compat_version`.

No run has a result file for this version.

## Ratchet

These rows are above `ratchet.toml` or are not in it. `rupg-compat report --raise` writes them to the ratchet.

| Level | Row | Ratchet | Now |
|---|---|---|---|
| L1 | 19 libpq_pipeline | none | 24 |
| L1 | 19 trace activerecord | none | 829 |
| L1 | 19 trace asyncpg | none | 140 |
| L1 | 19 trace diesel | none | 449 |
| L1 | 19 trace django | none | 112 |
| L1 | 19 trace drizzle | none | 425 |
| L1 | 19 trace ecto | none | 633 |
| L1 | 19 trace efcore | none | 2296 |
| L1 | 19 trace flyway | none | 1329 |
| L1 | 19 trace golang-migrate | none | 172 |
| L1 | 19 trace grafana | none | 176 |
| L1 | 19 trace hibernate | none | 402 |
| L1 | 19 trace lib-pq | none | 152 |
| L1 | 19 trace libpq | none | 124 |
| L1 | 19 trace liquibase | none | 1982 |
| L1 | 19 trace metabase | none | 593 |
| L1 | 19 trace node-postgres | none | 123 |
| L1 | 19 trace npgsql | none | 320 |
| L1 | 19 trace pg_dump | none | 20798 |
| L1 | 19 trace pgadmin4 | none | 384 |
| L1 | 19 trace pgbouncer | none | 1385 |
| L1 | 19 trace pgcat | none | 1381 |
| L1 | 19 trace pgjdbc | none | 146 |
| L1 | 19 trace pgx | none | 133 |
| L1 | 19 trace postgres-js | none | 445 |
| L1 | 19 trace postgrex | none | 359 |
| L1 | 19 trace prisma | none | 584 |
| L1 | 19 trace psql | none | 126 |
| L1 | 19 trace psqlodbc | none | 162 |
| L1 | 19 trace psycopg | none | 131 |
| L1 | 19 trace psycopg2 | none | 99 |
| L1 | 19 trace r2dbc-postgresql | none | 139 |
| L1 | 19 trace ruby-pg | none | 124 |
| L1 | 19 trace sequelize | none | 534 |
| L1 | 19 trace sqlalchemy | none | 412 |
| L1 | 19 trace sqlx | none | 146 |
| L1 | 19 trace supavisor | none | 1629 |
| L1 | 19 trace superset | none | 1569 |
| L1 | 19 trace tokio-postgres | none | 214 |
| L1 | 19 trace typeorm | none | 240 |
| L2 | 19 catalog relations | none | 190 |
| L2 | 19 catalog relations with the gate schema | none | 190 |
| L2 | 19 gate G1 | none | 6 |
| L2 | 19 pg_dump round trip | none | 6 |
| L2 | 19 rows of information_schema._pg_foreign_data_wrappers | none | 0 |
| L2 | 19 rows of information_schema._pg_foreign_data_wrappers with the gate schema | none | 0 |
| L2 | 19 rows of information_schema._pg_foreign_servers | none | 0 |
| L2 | 19 rows of information_schema._pg_foreign_servers with the gate schema | none | 0 |
| L2 | 19 rows of information_schema._pg_foreign_table_columns | none | 0 |
| L2 | 19 rows of information_schema._pg_foreign_table_columns with the gate schema | none | 0 |
| L2 | 19 rows of information_schema._pg_foreign_tables | none | 0 |
| L2 | 19 rows of information_schema._pg_foreign_tables with the gate schema | none | 0 |
| L2 | 19 rows of information_schema._pg_user_mappings | none | 0 |
| L2 | 19 rows of information_schema._pg_user_mappings with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.administrable_role_authorizations | none | 0 |
| L2 | 19 rows of information_schema.administrable_role_authorizations with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.applicable_roles | none | 4 |
| L2 | 19 rows of information_schema.applicable_roles with the gate schema | none | 4 |
| L2 | 19 rows of information_schema.attributes | none | 0 |
| L2 | 19 rows of information_schema.attributes with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.character_sets | none | 1 |
| L2 | 19 rows of information_schema.character_sets with the gate schema | none | 1 |
| L2 | 19 rows of information_schema.check_constraint_routine_usage | none | 0 |
| L2 | 19 rows of information_schema.check_constraint_routine_usage with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.check_constraints | none | 84 |
| L2 | 19 rows of information_schema.check_constraints with the gate schema | none | 104 |
| L2 | 19 rows of information_schema.collation_character_set_applicability | none | 858 |
| L2 | 19 rows of information_schema.collation_character_set_applicability with the gate schema | none | 858 |
| L2 | 19 rows of information_schema.collations | none | 858 |
| L2 | 19 rows of information_schema.collations with the gate schema | none | 858 |
| L2 | 19 rows of information_schema.column_column_usage | none | 0 |
| L2 | 19 rows of information_schema.column_column_usage with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.column_domain_usage | none | 683 |
| L2 | 19 rows of information_schema.column_domain_usage with the gate schema | none | 683 |
| L2 | 19 rows of information_schema.column_options | none | 0 |
| L2 | 19 rows of information_schema.column_options with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.column_privileges | none | 10958 |
| L2 | 19 rows of information_schema.column_privileges with the gate schema | none | 11166 |
| L2 | 19 rows of information_schema.column_udt_usage | none | 2217 |
| L2 | 19 rows of information_schema.column_udt_usage with the gate schema | none | 2269 |
| L2 | 19 rows of information_schema.columns | none | 2217 |
| L2 | 19 rows of information_schema.columns with the gate schema | none | 2269 |
| L2 | 19 rows of information_schema.constraint_column_usage | none | 263 |
| L2 | 19 rows of information_schema.constraint_column_usage with the gate schema | none | 293 |
| L2 | 19 rows of information_schema.constraint_table_usage | none | 110 |
| L2 | 19 rows of information_schema.constraint_table_usage with the gate schema | none | 119 |
| L2 | 19 rows of information_schema.data_type_privileges | none | 12411 |
| L2 | 19 rows of information_schema.data_type_privileges with the gate schema | none | 12463 |
| L2 | 19 rows of information_schema.domain_constraints | none | 2 |
| L2 | 19 rows of information_schema.domain_constraints with the gate schema | none | 2 |
| L2 | 19 rows of information_schema.domain_udt_usage | none | 5 |
| L2 | 19 rows of information_schema.domain_udt_usage with the gate schema | none | 5 |
| L2 | 19 rows of information_schema.domains | none | 5 |
| L2 | 19 rows of information_schema.domains with the gate schema | none | 5 |
| L2 | 19 rows of information_schema.element_types | none | 278 |
| L2 | 19 rows of information_schema.element_types with the gate schema | none | 279 |
| L2 | 19 rows of information_schema.enabled_roles | none | 17 |
| L2 | 19 rows of information_schema.enabled_roles with the gate schema | none | 17 |
| L2 | 19 rows of information_schema.foreign_data_wrapper_options | none | 0 |
| L2 | 19 rows of information_schema.foreign_data_wrapper_options with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.foreign_data_wrappers | none | 0 |
| L2 | 19 rows of information_schema.foreign_data_wrappers with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.foreign_server_options | none | 0 |
| L2 | 19 rows of information_schema.foreign_server_options with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.foreign_servers | none | 0 |
| L2 | 19 rows of information_schema.foreign_servers with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.foreign_table_options | none | 0 |
| L2 | 19 rows of information_schema.foreign_table_options with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.foreign_tables | none | 0 |
| L2 | 19 rows of information_schema.foreign_tables with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.information_schema_catalog_name | none | 1 |
| L2 | 19 rows of information_schema.information_schema_catalog_name with the gate schema | none | 1 |
| L2 | 19 rows of information_schema.key_column_usage | none | 181 |
| L2 | 19 rows of information_schema.key_column_usage with the gate schema | none | 191 |
| L2 | 19 rows of information_schema.parameters | none | 6759 |
| L2 | 19 rows of information_schema.parameters with the gate schema | none | 6759 |
| L2 | 19 rows of information_schema.referential_constraints | none | 0 |
| L2 | 19 rows of information_schema.referential_constraints with the gate schema | none | 3 |
| L2 | 19 rows of information_schema.role_column_grants | none | 10958 |
| L2 | 19 rows of information_schema.role_column_grants with the gate schema | none | 11166 |
| L2 | 19 rows of information_schema.role_routine_grants | none | 6809 |
| L2 | 19 rows of information_schema.role_routine_grants with the gate schema | none | 6809 |
| L2 | 19 rows of information_schema.role_table_grants | none | 1734 |
| L2 | 19 rows of information_schema.role_table_grants with the gate schema | none | 1776 |
| L2 | 19 rows of information_schema.role_udt_grants | none | 438 |
| L2 | 19 rows of information_schema.role_udt_grants with the gate schema | none | 450 |
| L2 | 19 rows of information_schema.role_usage_grants | none | 868 |
| L2 | 19 rows of information_schema.role_usage_grants with the gate schema | none | 868 |
| L2 | 19 rows of information_schema.routine_column_usage | none | 0 |
| L2 | 19 rows of information_schema.routine_column_usage with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.routine_privileges | none | 6809 |
| L2 | 19 rows of information_schema.routine_privileges with the gate schema | none | 6809 |
| L2 | 19 rows of information_schema.routine_routine_usage | none | 2 |
| L2 | 19 rows of information_schema.routine_routine_usage with the gate schema | none | 2 |
| L2 | 19 rows of information_schema.routine_sequence_usage | none | 0 |
| L2 | 19 rows of information_schema.routine_sequence_usage with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.routine_table_usage | none | 0 |
| L2 | 19 rows of information_schema.routine_table_usage with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.routines | none | 3430 |
| L2 | 19 rows of information_schema.routines with the gate schema | none | 3430 |
| L2 | 19 rows of information_schema.schemata | none | 4 |
| L2 | 19 rows of information_schema.schemata with the gate schema | none | 5 |
| L2 | 19 rows of information_schema.sequences | none | 0 |
| L2 | 19 rows of information_schema.sequences with the gate schema | none | 3 |
| L2 | 19 rows of information_schema.table_constraints | none | 192 |
| L2 | 19 rows of information_schema.table_constraints with the gate schema | none | 221 |
| L2 | 19 rows of information_schema.table_privileges | none | 1734 |
| L2 | 19 rows of information_schema.table_privileges with the gate schema | none | 1776 |
| L2 | 19 rows of information_schema.tables | none | 219 |
| L2 | 19 rows of information_schema.tables with the gate schema | none | 225 |
| L2 | 19 rows of information_schema.transforms | none | 0 |
| L2 | 19 rows of information_schema.transforms with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.triggered_update_columns | none | 0 |
| L2 | 19 rows of information_schema.triggered_update_columns with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.triggers | none | 0 |
| L2 | 19 rows of information_schema.triggers with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.udt_privileges | none | 438 |
| L2 | 19 rows of information_schema.udt_privileges with the gate schema | none | 450 |
| L2 | 19 rows of information_schema.usage_privileges | none | 868 |
| L2 | 19 rows of information_schema.usage_privileges with the gate schema | none | 868 |
| L2 | 19 rows of information_schema.user_defined_types | none | 0 |
| L2 | 19 rows of information_schema.user_defined_types with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.user_mapping_options | none | 0 |
| L2 | 19 rows of information_schema.user_mapping_options with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.user_mappings | none | 0 |
| L2 | 19 rows of information_schema.user_mappings with the gate schema | none | 0 |
| L2 | 19 rows of information_schema.view_column_usage | none | 292 |
| L2 | 19 rows of information_schema.view_column_usage with the gate schema | none | 297 |
| L2 | 19 rows of information_schema.view_routine_usage | none | 31 |
| L2 | 19 rows of information_schema.view_routine_usage with the gate schema | none | 31 |
| L2 | 19 rows of information_schema.view_table_usage | none | 42 |
| L2 | 19 rows of information_schema.view_table_usage with the gate schema | none | 44 |
| L2 | 19 rows of information_schema.views | none | 151 |
| L2 | 19 rows of information_schema.views with the gate schema | none | 152 |
| L2 | 19 rows of pg_catalog.pg_aggregate | none | 163 |
| L2 | 19 rows of pg_catalog.pg_aggregate with the gate schema | none | 163 |
| L2 | 19 rows of pg_catalog.pg_aios | none | 0 |
| L2 | 19 rows of pg_catalog.pg_aios with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_am | none | 7 |
| L2 | 19 rows of pg_catalog.pg_am with the gate schema | none | 7 |
| L2 | 19 rows of pg_catalog.pg_amop | none | 951 |
| L2 | 19 rows of pg_catalog.pg_amop with the gate schema | none | 951 |
| L2 | 19 rows of pg_catalog.pg_amproc | none | 720 |
| L2 | 19 rows of pg_catalog.pg_amproc with the gate schema | none | 720 |
| L2 | 19 rows of pg_catalog.pg_attrdef | none | 0 |
| L2 | 19 rows of pg_catalog.pg_attrdef with the gate schema | none | 7 |
| L2 | 19 rows of pg_catalog.pg_attribute | none | 3259 |
| L2 | 19 rows of pg_catalog.pg_attribute with the gate schema | none | 3424 |
| L2 | 19 rows of pg_catalog.pg_auth_members | none | 3 |
| L2 | 19 rows of pg_catalog.pg_auth_members with the gate schema | none | 3 |
| L2 | 19 rows of pg_catalog.pg_authid | none | 17 |
| L2 | 19 rows of pg_catalog.pg_authid with the gate schema | none | 17 |
| L2 | 19 rows of pg_catalog.pg_available_extension_versions | none | 1 |
| L2 | 19 rows of pg_catalog.pg_available_extension_versions with the gate schema | none | 1 |
| L2 | 19 rows of pg_catalog.pg_available_extensions | none | 1 |
| L2 | 19 rows of pg_catalog.pg_available_extensions with the gate schema | none | 1 |
| L2 | 19 rows of pg_catalog.pg_cast | none | 249 |
| L2 | 19 rows of pg_catalog.pg_cast with the gate schema | none | 249 |
| L2 | 19 rows of pg_catalog.pg_class | none | 421 |
| L2 | 19 rows of pg_catalog.pg_class with the gate schema | none | 448 |
| L2 | 19 rows of pg_catalog.pg_collation | none | 6 |
| L2 | 19 rows of pg_catalog.pg_collation with the gate schema | none | 6 |
| L2 | 19 rows of pg_catalog.pg_config | none | 23 |
| L2 | 19 rows of pg_catalog.pg_config with the gate schema | none | 23 |
| L2 | 19 rows of pg_catalog.pg_constraint | none | 194 |
| L2 | 19 rows of pg_catalog.pg_constraint with the gate schema | none | 223 |
| L2 | 19 rows of pg_catalog.pg_conversion | none | 98 |
| L2 | 19 rows of pg_catalog.pg_conversion with the gate schema | none | 98 |
| L2 | 19 rows of pg_catalog.pg_cursors | none | 0 |
| L2 | 19 rows of pg_catalog.pg_cursors with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_database | none | 10 |
| L2 | 19 rows of pg_catalog.pg_database with the gate schema | none | 10 |
| L2 | 19 rows of pg_catalog.pg_db_role_setting | none | 2 |
| L2 | 19 rows of pg_catalog.pg_db_role_setting with the gate schema | none | 2 |
| L2 | 19 rows of pg_catalog.pg_default_acl | none | 0 |
| L2 | 19 rows of pg_catalog.pg_default_acl with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_depend | none | 1770 |
| L2 | 19 rows of pg_catalog.pg_depend with the gate schema | none | 1887 |
| L2 | 19 rows of pg_catalog.pg_description | none | 5352 |
| L2 | 19 rows of pg_catalog.pg_description with the gate schema | none | 5356 |
| L2 | 19 rows of pg_catalog.pg_dsm_registry_allocations | none | 0 |
| L2 | 19 rows of pg_catalog.pg_dsm_registry_allocations with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_enum | none | 0 |
| L2 | 19 rows of pg_catalog.pg_enum with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_event_trigger | none | 0 |
| L2 | 19 rows of pg_catalog.pg_event_trigger with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_extension | none | 1 |
| L2 | 19 rows of pg_catalog.pg_extension with the gate schema | none | 1 |
| L2 | 19 rows of pg_catalog.pg_file_settings | none | 0 |
| L2 | 19 rows of pg_catalog.pg_file_settings with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_foreign_data_wrapper | none | 0 |
| L2 | 19 rows of pg_catalog.pg_foreign_data_wrapper with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_foreign_server | none | 0 |
| L2 | 19 rows of pg_catalog.pg_foreign_server with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_foreign_table | none | 0 |
| L2 | 19 rows of pg_catalog.pg_foreign_table with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_group | none | 16 |
| L2 | 19 rows of pg_catalog.pg_group with the gate schema | none | 16 |
| L2 | 19 rows of pg_catalog.pg_hba_file_rules | none | 3 |
| L2 | 19 rows of pg_catalog.pg_hba_file_rules with the gate schema | none | 3 |
| L2 | 19 rows of pg_catalog.pg_ident_file_mappings | none | 0 |
| L2 | 19 rows of pg_catalog.pg_ident_file_mappings with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_index | none | 163 |
| L2 | 19 rows of pg_catalog.pg_index with the gate schema | none | 177 |
| L2 | 19 rows of pg_catalog.pg_indexes | none | 124 |
| L2 | 19 rows of pg_catalog.pg_indexes with the gate schema | none | 134 |
| L2 | 19 rows of pg_catalog.pg_inherits | none | 0 |
| L2 | 19 rows of pg_catalog.pg_inherits with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_init_privs | none | 241 |
| L2 | 19 rows of pg_catalog.pg_init_privs with the gate schema | none | 241 |
| L2 | 19 rows of pg_catalog.pg_language | none | 4 |
| L2 | 19 rows of pg_catalog.pg_language with the gate schema | none | 4 |
| L2 | 19 rows of pg_catalog.pg_largeobject | none | 0 |
| L2 | 19 rows of pg_catalog.pg_largeobject with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_largeobject_metadata | none | 0 |
| L2 | 19 rows of pg_catalog.pg_largeobject_metadata with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_matviews | none | 0 |
| L2 | 19 rows of pg_catalog.pg_matviews with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_namespace | none | 4 |
| L2 | 19 rows of pg_catalog.pg_namespace with the gate schema | none | 5 |
| L2 | 19 rows of pg_catalog.pg_opclass | none | 179 |
| L2 | 19 rows of pg_catalog.pg_opclass with the gate schema | none | 179 |
| L2 | 19 rows of pg_catalog.pg_operator | none | 805 |
| L2 | 19 rows of pg_catalog.pg_operator with the gate schema | none | 805 |
| L2 | 19 rows of pg_catalog.pg_opfamily | none | 148 |
| L2 | 19 rows of pg_catalog.pg_opfamily with the gate schema | none | 148 |
| L2 | 19 rows of pg_catalog.pg_parameter_acl | none | 0 |
| L2 | 19 rows of pg_catalog.pg_parameter_acl with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_partitioned_table | none | 0 |
| L2 | 19 rows of pg_catalog.pg_partitioned_table with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_policies | none | 0 |
| L2 | 19 rows of pg_catalog.pg_policies with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_policy | none | 0 |
| L2 | 19 rows of pg_catalog.pg_policy with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_prepared_statements | none | 0 |
| L2 | 19 rows of pg_catalog.pg_prepared_statements with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_prepared_xacts | none | 0 |
| L2 | 19 rows of pg_catalog.pg_prepared_xacts with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_proc | none | 3430 |
| L2 | 19 rows of pg_catalog.pg_proc with the gate schema | none | 3430 |
| L2 | 19 rows of pg_catalog.pg_publication | none | 0 |
| L2 | 19 rows of pg_catalog.pg_publication with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_publication_namespace | none | 0 |
| L2 | 19 rows of pg_catalog.pg_publication_namespace with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_publication_rel | none | 0 |
| L2 | 19 rows of pg_catalog.pg_publication_rel with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_publication_sequences | none | 0 |
| L2 | 19 rows of pg_catalog.pg_publication_sequences with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_publication_tables | none | 0 |
| L2 | 19 rows of pg_catalog.pg_publication_tables with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_range | none | 6 |
| L2 | 19 rows of pg_catalog.pg_range with the gate schema | none | 6 |
| L2 | 19 rows of pg_catalog.pg_replication_origin | none | 0 |
| L2 | 19 rows of pg_catalog.pg_replication_origin with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_replication_origin_status | none | 0 |
| L2 | 19 rows of pg_catalog.pg_replication_origin_status with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_replication_slots | none | 0 |
| L2 | 19 rows of pg_catalog.pg_replication_slots with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_rewrite | none | 153 |
| L2 | 19 rows of pg_catalog.pg_rewrite with the gate schema | none | 154 |
| L2 | 19 rows of pg_catalog.pg_roles | none | 17 |
| L2 | 19 rows of pg_catalog.pg_roles with the gate schema | none | 17 |
| L2 | 19 rows of pg_catalog.pg_rules | none | 2 |
| L2 | 19 rows of pg_catalog.pg_rules with the gate schema | none | 2 |
| L2 | 19 rows of pg_catalog.pg_seclabel | none | 0 |
| L2 | 19 rows of pg_catalog.pg_seclabel with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_seclabels | none | 0 |
| L2 | 19 rows of pg_catalog.pg_seclabels with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_sequence | none | 0 |
| L2 | 19 rows of pg_catalog.pg_sequence with the gate schema | none | 3 |
| L2 | 19 rows of pg_catalog.pg_sequences | none | 0 |
| L2 | 19 rows of pg_catalog.pg_sequences with the gate schema | none | 3 |
| L2 | 19 rows of pg_catalog.pg_settings | none | 398 |
| L2 | 19 rows of pg_catalog.pg_settings with the gate schema | none | 398 |
| L2 | 19 rows of pg_catalog.pg_shadow | none | 1 |
| L2 | 19 rows of pg_catalog.pg_shadow with the gate schema | none | 1 |
| L2 | 19 rows of pg_catalog.pg_shdepend | none | 0 |
| L2 | 19 rows of pg_catalog.pg_shdepend with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_shdescription | none | 3 |
| L2 | 19 rows of pg_catalog.pg_shdescription with the gate schema | none | 3 |
| L2 | 19 rows of pg_catalog.pg_shmem_allocations | none | 0 |
| L2 | 19 rows of pg_catalog.pg_shmem_allocations with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_shmem_allocations_numa | none | 0 |
| L2 | 19 rows of pg_catalog.pg_shmem_allocations_numa with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_shseclabel | none | 0 |
| L2 | 19 rows of pg_catalog.pg_shseclabel with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_activity | none | 1 |
| L2 | 19 rows of pg_catalog.pg_stat_activity with the gate schema | none | 1 |
| L2 | 19 rows of pg_catalog.pg_stat_archiver | none | 1 |
| L2 | 19 rows of pg_catalog.pg_stat_archiver with the gate schema | none | 1 |
| L2 | 19 rows of pg_catalog.pg_stat_autovacuum_scores | none | 107 |
| L2 | 19 rows of pg_catalog.pg_stat_autovacuum_scores with the gate schema | none | 116 |
| L2 | 19 rows of pg_catalog.pg_stat_bgwriter | none | 1 |
| L2 | 19 rows of pg_catalog.pg_stat_bgwriter with the gate schema | none | 1 |
| L2 | 19 rows of pg_catalog.pg_stat_checkpointer | none | 1 |
| L2 | 19 rows of pg_catalog.pg_stat_checkpointer with the gate schema | none | 1 |
| L2 | 19 rows of pg_catalog.pg_stat_database | none | 10 |
| L2 | 19 rows of pg_catalog.pg_stat_database with the gate schema | none | 10 |
| L2 | 19 rows of pg_catalog.pg_stat_database_conflicts | none | 10 |
| L2 | 19 rows of pg_catalog.pg_stat_database_conflicts with the gate schema | none | 10 |
| L2 | 19 rows of pg_catalog.pg_stat_gssapi | none | 1 |
| L2 | 19 rows of pg_catalog.pg_stat_gssapi with the gate schema | none | 1 |
| L2 | 19 rows of pg_catalog.pg_stat_io | none | 79 |
| L2 | 19 rows of pg_catalog.pg_stat_io with the gate schema | none | 79 |
| L2 | 19 rows of pg_catalog.pg_stat_lock | none | 12 |
| L2 | 19 rows of pg_catalog.pg_stat_lock with the gate schema | none | 12 |
| L2 | 19 rows of pg_catalog.pg_stat_progress_analyze | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_progress_analyze with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_progress_basebackup | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_progress_basebackup with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_progress_cluster | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_progress_cluster with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_progress_copy | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_progress_copy with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_progress_create_index | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_progress_create_index with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_progress_repack | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_progress_repack with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_progress_vacuum | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_progress_vacuum with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_recovery | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_recovery with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_recovery_prefetch | none | 1 |
| L2 | 19 rows of pg_catalog.pg_stat_recovery_prefetch with the gate schema | none | 1 |
| L2 | 19 rows of pg_catalog.pg_stat_replication | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_replication with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_replication_slots | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_replication_slots with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_slru | none | 8 |
| L2 | 19 rows of pg_catalog.pg_stat_slru with the gate schema | none | 8 |
| L2 | 19 rows of pg_catalog.pg_stat_ssl | none | 1 |
| L2 | 19 rows of pg_catalog.pg_stat_ssl with the gate schema | none | 1 |
| L2 | 19 rows of pg_catalog.pg_stat_subscription | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_subscription with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_subscription_stats | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_subscription_stats with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_user_functions | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_user_functions with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_user_indexes | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_user_indexes with the gate schema | none | 10 |
| L2 | 19 rows of pg_catalog.pg_stat_user_tables | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_user_tables with the gate schema | none | 5 |
| L2 | 19 rows of pg_catalog.pg_stat_wal | none | 1 |
| L2 | 19 rows of pg_catalog.pg_stat_wal with the gate schema | none | 1 |
| L2 | 19 rows of pg_catalog.pg_stat_wal_receiver | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_wal_receiver with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_xact_all_tables | none | 107 |
| L2 | 19 rows of pg_catalog.pg_stat_xact_all_tables with the gate schema | none | 116 |
| L2 | 19 rows of pg_catalog.pg_stat_xact_sys_tables | none | 107 |
| L2 | 19 rows of pg_catalog.pg_stat_xact_sys_tables with the gate schema | none | 111 |
| L2 | 19 rows of pg_catalog.pg_stat_xact_user_functions | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_xact_user_functions with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_xact_user_tables | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stat_xact_user_tables with the gate schema | none | 5 |
| L2 | 19 rows of pg_catalog.pg_statio_all_indexes | none | 163 |
| L2 | 19 rows of pg_catalog.pg_statio_all_indexes with the gate schema | none | 177 |
| L2 | 19 rows of pg_catalog.pg_statio_all_sequences | none | 0 |
| L2 | 19 rows of pg_catalog.pg_statio_all_sequences with the gate schema | none | 3 |
| L2 | 19 rows of pg_catalog.pg_statio_all_tables | none | 106 |
| L2 | 19 rows of pg_catalog.pg_statio_all_tables with the gate schema | none | 115 |
| L2 | 19 rows of pg_catalog.pg_statio_sys_indexes | none | 163 |
| L2 | 19 rows of pg_catalog.pg_statio_sys_indexes with the gate schema | none | 167 |
| L2 | 19 rows of pg_catalog.pg_statio_sys_sequences | none | 0 |
| L2 | 19 rows of pg_catalog.pg_statio_sys_sequences with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_statio_sys_tables | none | 106 |
| L2 | 19 rows of pg_catalog.pg_statio_sys_tables with the gate schema | none | 110 |
| L2 | 19 rows of pg_catalog.pg_statio_user_indexes | none | 0 |
| L2 | 19 rows of pg_catalog.pg_statio_user_indexes with the gate schema | none | 10 |
| L2 | 19 rows of pg_catalog.pg_statio_user_sequences | none | 0 |
| L2 | 19 rows of pg_catalog.pg_statio_user_sequences with the gate schema | none | 3 |
| L2 | 19 rows of pg_catalog.pg_statio_user_tables | none | 0 |
| L2 | 19 rows of pg_catalog.pg_statio_user_tables with the gate schema | none | 5 |
| L2 | 19 rows of pg_catalog.pg_statistic | none | 0 |
| L2 | 19 rows of pg_catalog.pg_statistic with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_statistic_ext | none | 0 |
| L2 | 19 rows of pg_catalog.pg_statistic_ext with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_statistic_ext_data | none | 0 |
| L2 | 19 rows of pg_catalog.pg_statistic_ext_data with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stats | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stats with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stats_ext | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stats_ext with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stats_ext_exprs | none | 0 |
| L2 | 19 rows of pg_catalog.pg_stats_ext_exprs with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_subscription | none | 0 |
| L2 | 19 rows of pg_catalog.pg_subscription with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_subscription_rel | none | 0 |
| L2 | 19 rows of pg_catalog.pg_subscription_rel with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_tables | none | 68 |
| L2 | 19 rows of pg_catalog.pg_tables with the gate schema | none | 73 |
| L2 | 19 rows of pg_catalog.pg_tablespace | none | 2 |
| L2 | 19 rows of pg_catalog.pg_tablespace with the gate schema | none | 2 |
| L2 | 19 rows of pg_catalog.pg_timezone_abbrevs | none | 195 |
| L2 | 19 rows of pg_catalog.pg_timezone_abbrevs with the gate schema | none | 195 |
| L2 | 19 rows of pg_catalog.pg_timezone_names | none | 598 |
| L2 | 19 rows of pg_catalog.pg_timezone_names with the gate schema | none | 598 |
| L2 | 19 rows of pg_catalog.pg_transform | none | 0 |
| L2 | 19 rows of pg_catalog.pg_transform with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_trigger | none | 0 |
| L2 | 19 rows of pg_catalog.pg_trigger with the gate schema | none | 12 |
| L2 | 19 rows of pg_catalog.pg_ts_config | none | 32 |
| L2 | 19 rows of pg_catalog.pg_ts_config with the gate schema | none | 32 |
| L2 | 19 rows of pg_catalog.pg_ts_config_map | none | 608 |
| L2 | 19 rows of pg_catalog.pg_ts_config_map with the gate schema | none | 608 |
| L2 | 19 rows of pg_catalog.pg_ts_dict | none | 32 |
| L2 | 19 rows of pg_catalog.pg_ts_dict with the gate schema | none | 32 |
| L2 | 19 rows of pg_catalog.pg_ts_parser | none | 1 |
| L2 | 19 rows of pg_catalog.pg_ts_parser with the gate schema | none | 1 |
| L2 | 19 rows of pg_catalog.pg_ts_template | none | 5 |
| L2 | 19 rows of pg_catalog.pg_ts_template with the gate schema | none | 5 |
| L2 | 19 rows of pg_catalog.pg_type | none | 637 |
| L2 | 19 rows of pg_catalog.pg_type with the gate schema | none | 649 |
| L2 | 19 rows of pg_catalog.pg_user | none | 1 |
| L2 | 19 rows of pg_catalog.pg_user with the gate schema | none | 1 |
| L2 | 19 rows of pg_catalog.pg_user_mapping | none | 0 |
| L2 | 19 rows of pg_catalog.pg_user_mapping with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_user_mappings | none | 0 |
| L2 | 19 rows of pg_catalog.pg_user_mappings with the gate schema | none | 0 |
| L2 | 19 rows of pg_catalog.pg_views | none | 151 |
| L2 | 19 rows of pg_catalog.pg_views with the gate schema | none | 152 |
| L2 | 19 rows of pg_catalog.pg_wait_events | none | 287 |
| L2 | 19 rows of pg_catalog.pg_wait_events with the gate schema | none | 287 |
| L3 | 19 regression tests | none | 63 |
| L4 | 19 isolation specs | none | 18 |
| L4 | 19 parameter cases | none | 1298 |
