# rupg-compat report 2026-10-27

This report reads 0 result files in `/home/runner/work/rupg-compat/rupg-compat/work/results`. Each number is the count that is equal to the oracle, of the count that the oracle gives. The server column is the server under test, and the oracle column is the oracle against itself. The unstable cases of the oracle are not counted (spec/21 section 21.1). The ratchet checks the count of the oracle less the differences, with each unstable case as no difference, so that a run with more unstable cases does not fail it. A relation that was unstable in the run of the server under test shows as unstable, and the ratchet does not check its rows in that run.

## Pins

| Pin | Release | Commit |
|---|---|---|
| PostgreSQL 14 | 14.24 | `6b3806732b7c5df06bdd0ed150e8a07b4ad62315` |
| PostgreSQL 15 | 15.19 | `2ff1375b5dd8bf09d8cb0e795974528180fd75ca` |
| PostgreSQL 16 | 16.15 | `7d3e000c5961a544302072058a1184e9a588837b` |
| PostgreSQL 17 | 17.11 | `083ac033419f690758508e08c1736089384bbee8` |
| PostgreSQL 18 | 18.6 | `724edf9bde9d356724ad384a2e196edc3c9f80f7` |
| PostgreSQL 19 | 19beta4 | `7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333` |
| rupg | | `57a3e9812d2cf8afff9c067d9c4cbca2ad6ddb8e` |

| Client | Version |
|---|---|
| activerecord | 8.1.4 |
| asyncpg | 0.32.0 |
| beekeeper | 6.1.6 |
| dbeaver | 26.2.3 |
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

No run has a result file for this version.

## Ratchet

These rows are below `ratchet.toml`. A number may go down only with an entry in spec/24 of tamnd/rupg that says why.

| Level | Row | Ratchet | Now |
|---|---|---|---|
| L1 | 19 connect with SCRAM over TLS | 17 | no result |
| L1 | 19 libpq_pipeline | 24 | no result |
| L1 | 19 trace activerecord | 829 | no result |
| L1 | 19 trace asyncpg | 140 | no result |
| L1 | 19 trace beekeeper | 506 | no result |
| L1 | 19 trace dbeaver | 2103 | no result |
| L1 | 19 trace diesel | 449 | no result |
| L1 | 19 trace django | 112 | no result |
| L1 | 19 trace drizzle | 431 | no result |
| L1 | 19 trace ecto | 633 | no result |
| L1 | 19 trace efcore | 4010 | no result |
| L1 | 19 trace flyway | 1329 | no result |
| L1 | 19 trace golang-migrate | 172 | no result |
| L1 | 19 trace grafana | 176 | no result |
| L1 | 19 trace hibernate | 402 | no result |
| L1 | 19 trace lib-pq | 152 | no result |
| L1 | 19 trace libpq | 124 | no result |
| L1 | 19 trace liquibase | 2010 | no result |
| L1 | 19 trace metabase | 593 | no result |
| L1 | 19 trace node-postgres | 123 | no result |
| L1 | 19 trace npgsql | 320 | no result |
| L1 | 19 trace pg_dump | 22528 | no result |
| L1 | 19 trace pgadmin4 | 392 | no result |
| L1 | 19 trace pgbouncer | 1385 | no result |
| L1 | 19 trace pgcat | 1381 | no result |
| L1 | 19 trace pgjdbc | 146 | no result |
| L1 | 19 trace pgx | 133 | no result |
| L1 | 19 trace postgres-js | 445 | no result |
| L1 | 19 trace postgrex | 359 | no result |
| L1 | 19 trace prisma | 585 | no result |
| L1 | 19 trace psql | 126 | no result |
| L1 | 19 trace psqlodbc | 162 | no result |
| L1 | 19 trace psycopg | 131 | no result |
| L1 | 19 trace psycopg2 | 99 | no result |
| L1 | 19 trace r2dbc-postgresql | 139 | no result |
| L1 | 19 trace ruby-pg | 124 | no result |
| L1 | 19 trace sequelize | 538 | no result |
| L1 | 19 trace sqlalchemy | 412 | no result |
| L1 | 19 trace sqlx | 146 | no result |
| L1 | 19 trace supavisor | 1629 | no result |
| L1 | 19 trace superset | 1569 | no result |
| L1 | 19 trace tokio-postgres | 214 | no result |
| L1 | 19 trace typeorm | 240 | no result |
| L2 | 19 L2 tier 1 rows | 4321 | no result |
| L2 | 19 L2 tier 1 rows with the gate schema | 4526 | no result |
| L2 | 19 L2 tier 2 rows | 82093 | no result |
| L2 | 19 L2 tier 2 rows with the gate schema | 82947 | no result |
| L2 | 19 catalog queries of the client traces | 290 | no result |
| L2 | 19 catalog relations | 215 | no result |
| L2 | 19 catalog relations with the gate schema | 215 | no result |
| L2 | 19 gate G1 | 8 | no result |
| L2 | 19 pg_dump round trip | 7 | no result |
| L2 | 19 rows of information_schema._pg_foreign_data_wrappers | 0 | no result |
| L2 | 19 rows of information_schema._pg_foreign_data_wrappers with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema._pg_foreign_servers | 0 | no result |
| L2 | 19 rows of information_schema._pg_foreign_servers with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema._pg_foreign_table_columns | 0 | no result |
| L2 | 19 rows of information_schema._pg_foreign_table_columns with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema._pg_foreign_tables | 0 | no result |
| L2 | 19 rows of information_schema._pg_foreign_tables with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema._pg_user_mappings | 0 | no result |
| L2 | 19 rows of information_schema._pg_user_mappings with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.administrable_role_authorizations | 0 | no result |
| L2 | 19 rows of information_schema.administrable_role_authorizations with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.applicable_roles | 4 | no result |
| L2 | 19 rows of information_schema.applicable_roles with the gate schema | 4 | no result |
| L2 | 19 rows of information_schema.attributes | 0 | no result |
| L2 | 19 rows of information_schema.attributes with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.character_sets | 1 | no result |
| L2 | 19 rows of information_schema.character_sets with the gate schema | 1 | no result |
| L2 | 19 rows of information_schema.check_constraint_routine_usage | 0 | no result |
| L2 | 19 rows of information_schema.check_constraint_routine_usage with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.check_constraints | 84 | no result |
| L2 | 19 rows of information_schema.check_constraints with the gate schema | 104 | no result |
| L2 | 19 rows of information_schema.collation_character_set_applicability | 862 | no result |
| L2 | 19 rows of information_schema.collation_character_set_applicability with the gate schema | 862 | no result |
| L2 | 19 rows of information_schema.collations | 862 | no result |
| L2 | 19 rows of information_schema.collations with the gate schema | 862 | no result |
| L2 | 19 rows of information_schema.column_column_usage | 0 | no result |
| L2 | 19 rows of information_schema.column_column_usage with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.column_domain_usage | 683 | no result |
| L2 | 19 rows of information_schema.column_domain_usage with the gate schema | 683 | no result |
| L2 | 19 rows of information_schema.column_options | 0 | no result |
| L2 | 19 rows of information_schema.column_options with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.column_privileges | 10958 | no result |
| L2 | 19 rows of information_schema.column_privileges with the gate schema | 11166 | no result |
| L2 | 19 rows of information_schema.column_udt_usage | 2217 | no result |
| L2 | 19 rows of information_schema.column_udt_usage with the gate schema | 2269 | no result |
| L2 | 19 rows of information_schema.columns | 2217 | no result |
| L2 | 19 rows of information_schema.columns with the gate schema | 2269 | no result |
| L2 | 19 rows of information_schema.constraint_column_usage | 263 | no result |
| L2 | 19 rows of information_schema.constraint_column_usage with the gate schema | 293 | no result |
| L2 | 19 rows of information_schema.constraint_table_usage | 110 | no result |
| L2 | 19 rows of information_schema.constraint_table_usage with the gate schema | 119 | no result |
| L2 | 19 rows of information_schema.data_type_privileges | 12411 | no result |
| L2 | 19 rows of information_schema.data_type_privileges with the gate schema | 12463 | no result |
| L2 | 19 rows of information_schema.domain_constraints | 2 | no result |
| L2 | 19 rows of information_schema.domain_constraints with the gate schema | 2 | no result |
| L2 | 19 rows of information_schema.domain_udt_usage | 5 | no result |
| L2 | 19 rows of information_schema.domain_udt_usage with the gate schema | 5 | no result |
| L2 | 19 rows of information_schema.domains | 5 | no result |
| L2 | 19 rows of information_schema.domains with the gate schema | 5 | no result |
| L2 | 19 rows of information_schema.element_types | 278 | no result |
| L2 | 19 rows of information_schema.element_types with the gate schema | 279 | no result |
| L2 | 19 rows of information_schema.enabled_roles | 17 | no result |
| L2 | 19 rows of information_schema.enabled_roles with the gate schema | 17 | no result |
| L2 | 19 rows of information_schema.foreign_data_wrapper_options | 0 | no result |
| L2 | 19 rows of information_schema.foreign_data_wrapper_options with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.foreign_data_wrappers | 0 | no result |
| L2 | 19 rows of information_schema.foreign_data_wrappers with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.foreign_server_options | 0 | no result |
| L2 | 19 rows of information_schema.foreign_server_options with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.foreign_servers | 0 | no result |
| L2 | 19 rows of information_schema.foreign_servers with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.foreign_table_options | 0 | no result |
| L2 | 19 rows of information_schema.foreign_table_options with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.foreign_tables | 0 | no result |
| L2 | 19 rows of information_schema.foreign_tables with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.information_schema_catalog_name | 1 | no result |
| L2 | 19 rows of information_schema.information_schema_catalog_name with the gate schema | 1 | no result |
| L2 | 19 rows of information_schema.key_column_usage | 181 | no result |
| L2 | 19 rows of information_schema.key_column_usage with the gate schema | 191 | no result |
| L2 | 19 rows of information_schema.parameters | 6759 | no result |
| L2 | 19 rows of information_schema.parameters with the gate schema | 6759 | no result |
| L2 | 19 rows of information_schema.referential_constraints | 0 | no result |
| L2 | 19 rows of information_schema.referential_constraints with the gate schema | 3 | no result |
| L2 | 19 rows of information_schema.role_column_grants | 10958 | no result |
| L2 | 19 rows of information_schema.role_column_grants with the gate schema | 11166 | no result |
| L2 | 19 rows of information_schema.role_routine_grants | 6809 | no result |
| L2 | 19 rows of information_schema.role_routine_grants with the gate schema | 6809 | no result |
| L2 | 19 rows of information_schema.role_table_grants | 1734 | no result |
| L2 | 19 rows of information_schema.role_table_grants with the gate schema | 1776 | no result |
| L2 | 19 rows of information_schema.role_udt_grants | 438 | no result |
| L2 | 19 rows of information_schema.role_udt_grants with the gate schema | 450 | no result |
| L2 | 19 rows of information_schema.role_usage_grants | 872 | no result |
| L2 | 19 rows of information_schema.role_usage_grants with the gate schema | 872 | no result |
| L2 | 19 rows of information_schema.routine_column_usage | 0 | no result |
| L2 | 19 rows of information_schema.routine_column_usage with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.routine_privileges | 6809 | no result |
| L2 | 19 rows of information_schema.routine_privileges with the gate schema | 6809 | no result |
| L2 | 19 rows of information_schema.routine_routine_usage | 2 | no result |
| L2 | 19 rows of information_schema.routine_routine_usage with the gate schema | 2 | no result |
| L2 | 19 rows of information_schema.routine_sequence_usage | 0 | no result |
| L2 | 19 rows of information_schema.routine_sequence_usage with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.routine_table_usage | 0 | no result |
| L2 | 19 rows of information_schema.routine_table_usage with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.routines | 3430 | no result |
| L2 | 19 rows of information_schema.routines with the gate schema | 3430 | no result |
| L2 | 19 rows of information_schema.schemata | 4 | no result |
| L2 | 19 rows of information_schema.schemata with the gate schema | 5 | no result |
| L2 | 19 rows of information_schema.sequences | 0 | no result |
| L2 | 19 rows of information_schema.sequences with the gate schema | 3 | no result |
| L2 | 19 rows of information_schema.table_constraints | 192 | no result |
| L2 | 19 rows of information_schema.table_constraints with the gate schema | 221 | no result |
| L2 | 19 rows of information_schema.table_privileges | 1734 | no result |
| L2 | 19 rows of information_schema.table_privileges with the gate schema | 1776 | no result |
| L2 | 19 rows of information_schema.tables | 219 | no result |
| L2 | 19 rows of information_schema.tables with the gate schema | 225 | no result |
| L2 | 19 rows of information_schema.transforms | 0 | no result |
| L2 | 19 rows of information_schema.transforms with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.triggered_update_columns | 0 | no result |
| L2 | 19 rows of information_schema.triggered_update_columns with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.triggers | 0 | no result |
| L2 | 19 rows of information_schema.triggers with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.udt_privileges | 438 | no result |
| L2 | 19 rows of information_schema.udt_privileges with the gate schema | 450 | no result |
| L2 | 19 rows of information_schema.usage_privileges | 872 | no result |
| L2 | 19 rows of information_schema.usage_privileges with the gate schema | 872 | no result |
| L2 | 19 rows of information_schema.user_defined_types | 0 | no result |
| L2 | 19 rows of information_schema.user_defined_types with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.user_mapping_options | 0 | no result |
| L2 | 19 rows of information_schema.user_mapping_options with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.user_mappings | 0 | no result |
| L2 | 19 rows of information_schema.user_mappings with the gate schema | 0 | no result |
| L2 | 19 rows of information_schema.view_column_usage | 292 | no result |
| L2 | 19 rows of information_schema.view_column_usage with the gate schema | 297 | no result |
| L2 | 19 rows of information_schema.view_routine_usage | 31 | no result |
| L2 | 19 rows of information_schema.view_routine_usage with the gate schema | 31 | no result |
| L2 | 19 rows of information_schema.view_table_usage | 42 | no result |
| L2 | 19 rows of information_schema.view_table_usage with the gate schema | 44 | no result |
| L2 | 19 rows of information_schema.views | 151 | no result |
| L2 | 19 rows of information_schema.views with the gate schema | 152 | no result |
| L2 | 19 rows of pg_catalog.pg_aggregate | 163 | no result |
| L2 | 19 rows of pg_catalog.pg_aggregate with the gate schema | 163 | no result |
| L2 | 19 rows of pg_catalog.pg_aios | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_aios with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_am | 7 | no result |
| L2 | 19 rows of pg_catalog.pg_am with the gate schema | 7 | no result |
| L2 | 19 rows of pg_catalog.pg_amop | 951 | no result |
| L2 | 19 rows of pg_catalog.pg_amop with the gate schema | 951 | no result |
| L2 | 19 rows of pg_catalog.pg_amproc | 720 | no result |
| L2 | 19 rows of pg_catalog.pg_amproc with the gate schema | 720 | no result |
| L2 | 19 rows of pg_catalog.pg_attrdef | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_attrdef with the gate schema | 7 | no result |
| L2 | 19 rows of pg_catalog.pg_attribute | 3259 | no result |
| L2 | 19 rows of pg_catalog.pg_attribute with the gate schema | 3424 | no result |
| L2 | 19 rows of pg_catalog.pg_auth_members | 3 | no result |
| L2 | 19 rows of pg_catalog.pg_auth_members with the gate schema | 3 | no result |
| L2 | 19 rows of pg_catalog.pg_authid | 17 | no result |
| L2 | 19 rows of pg_catalog.pg_authid with the gate schema | 17 | no result |
| L2 | 19 rows of pg_catalog.pg_available_extension_versions | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_available_extension_versions with the gate schema | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_available_extensions | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_available_extensions with the gate schema | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_cast | 249 | no result |
| L2 | 19 rows of pg_catalog.pg_cast with the gate schema | 249 | no result |
| L2 | 19 rows of pg_catalog.pg_class | 421 | no result |
| L2 | 19 rows of pg_catalog.pg_class with the gate schema | 448 | no result |
| L2 | 19 rows of pg_catalog.pg_collation | 862 | no result |
| L2 | 19 rows of pg_catalog.pg_collation with the gate schema | 862 | no result |
| L2 | 19 rows of pg_catalog.pg_config | 23 | no result |
| L2 | 19 rows of pg_catalog.pg_config with the gate schema | 23 | no result |
| L2 | 19 rows of pg_catalog.pg_constraint | 194 | no result |
| L2 | 19 rows of pg_catalog.pg_constraint with the gate schema | 223 | no result |
| L2 | 19 rows of pg_catalog.pg_conversion | 98 | no result |
| L2 | 19 rows of pg_catalog.pg_conversion with the gate schema | 98 | no result |
| L2 | 19 rows of pg_catalog.pg_cursors | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_cursors with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_database | 5 | no result |
| L2 | 19 rows of pg_catalog.pg_database with the gate schema | 5 | no result |
| L2 | 19 rows of pg_catalog.pg_db_role_setting | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_db_role_setting with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_default_acl | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_default_acl with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_depend | 1770 | no result |
| L2 | 19 rows of pg_catalog.pg_depend with the gate schema | 1887 | no result |
| L2 | 19 rows of pg_catalog.pg_description | 5353 | no result |
| L2 | 19 rows of pg_catalog.pg_description with the gate schema | 5357 | no result |
| L2 | 19 rows of pg_catalog.pg_dsm_registry_allocations | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_dsm_registry_allocations with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_enum | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_enum with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_event_trigger | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_event_trigger with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_extension | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_extension with the gate schema | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_file_settings | 31 | no result |
| L2 | 19 rows of pg_catalog.pg_file_settings with the gate schema | 31 | no result |
| L2 | 19 rows of pg_catalog.pg_foreign_data_wrapper | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_foreign_data_wrapper with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_foreign_server | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_foreign_server with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_foreign_table | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_foreign_table with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_group | 16 | no result |
| L2 | 19 rows of pg_catalog.pg_group with the gate schema | 16 | no result |
| L2 | 19 rows of pg_catalog.pg_hba_file_rules | 3 | no result |
| L2 | 19 rows of pg_catalog.pg_hba_file_rules with the gate schema | 3 | no result |
| L2 | 19 rows of pg_catalog.pg_ident_file_mappings | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_ident_file_mappings with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_index | 163 | no result |
| L2 | 19 rows of pg_catalog.pg_index with the gate schema | 177 | no result |
| L2 | 19 rows of pg_catalog.pg_indexes | 124 | no result |
| L2 | 19 rows of pg_catalog.pg_indexes with the gate schema | 134 | no result |
| L2 | 19 rows of pg_catalog.pg_inherits | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_inherits with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_init_privs | 241 | no result |
| L2 | 19 rows of pg_catalog.pg_init_privs with the gate schema | 241 | no result |
| L2 | 19 rows of pg_catalog.pg_language | 4 | no result |
| L2 | 19 rows of pg_catalog.pg_language with the gate schema | 4 | no result |
| L2 | 19 rows of pg_catalog.pg_largeobject | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_largeobject with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_largeobject_metadata | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_largeobject_metadata with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_matviews | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_matviews with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_namespace | 4 | no result |
| L2 | 19 rows of pg_catalog.pg_namespace with the gate schema | 5 | no result |
| L2 | 19 rows of pg_catalog.pg_opclass | 179 | no result |
| L2 | 19 rows of pg_catalog.pg_opclass with the gate schema | 179 | no result |
| L2 | 19 rows of pg_catalog.pg_operator | 805 | no result |
| L2 | 19 rows of pg_catalog.pg_operator with the gate schema | 805 | no result |
| L2 | 19 rows of pg_catalog.pg_opfamily | 148 | no result |
| L2 | 19 rows of pg_catalog.pg_opfamily with the gate schema | 148 | no result |
| L2 | 19 rows of pg_catalog.pg_parameter_acl | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_parameter_acl with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_partitioned_table | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_partitioned_table with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_policies | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_policies with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_policy | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_policy with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_prepared_statements | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_prepared_statements with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_prepared_xacts | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_prepared_xacts with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_proc | 3430 | no result |
| L2 | 19 rows of pg_catalog.pg_proc with the gate schema | 3430 | no result |
| L2 | 19 rows of pg_catalog.pg_publication | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_publication with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_publication_namespace | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_publication_namespace with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_publication_rel | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_publication_rel with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_publication_sequences | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_publication_sequences with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_publication_tables | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_publication_tables with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_range | 6 | no result |
| L2 | 19 rows of pg_catalog.pg_range with the gate schema | 6 | no result |
| L2 | 19 rows of pg_catalog.pg_replication_origin | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_replication_origin with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_replication_origin_status | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_replication_origin_status with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_replication_slots | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_replication_slots with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_rewrite | 153 | no result |
| L2 | 19 rows of pg_catalog.pg_rewrite with the gate schema | 154 | no result |
| L2 | 19 rows of pg_catalog.pg_roles | 17 | no result |
| L2 | 19 rows of pg_catalog.pg_roles with the gate schema | 17 | no result |
| L2 | 19 rows of pg_catalog.pg_rules | 2 | no result |
| L2 | 19 rows of pg_catalog.pg_rules with the gate schema | 2 | no result |
| L2 | 19 rows of pg_catalog.pg_seclabel | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_seclabel with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_seclabels | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_seclabels with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_sequence | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_sequence with the gate schema | 3 | no result |
| L2 | 19 rows of pg_catalog.pg_sequences | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_sequences with the gate schema | 3 | no result |
| L2 | 19 rows of pg_catalog.pg_settings | 419 | no result |
| L2 | 19 rows of pg_catalog.pg_settings with the gate schema | 419 | no result |
| L2 | 19 rows of pg_catalog.pg_shadow | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_shadow with the gate schema | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_shdepend | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_shdepend with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_shdescription | 3 | no result |
| L2 | 19 rows of pg_catalog.pg_shdescription with the gate schema | 3 | no result |
| L2 | 19 rows of pg_catalog.pg_shmem_allocations | 79 | no result |
| L2 | 19 rows of pg_catalog.pg_shmem_allocations with the gate schema | 79 | no result |
| L2 | 19 rows of pg_catalog.pg_shmem_allocations_numa | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_shmem_allocations_numa with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_shseclabel | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_shseclabel with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_activity | 8 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_activity with the gate schema | 8 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_archiver | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_archiver with the gate schema | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_autovacuum_scores | 107 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_autovacuum_scores with the gate schema | 116 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_bgwriter | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_bgwriter with the gate schema | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_checkpointer | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_checkpointer with the gate schema | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_database | 6 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_database with the gate schema | 6 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_database_conflicts | 5 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_database_conflicts with the gate schema | 5 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_gssapi | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_gssapi with the gate schema | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_io | 79 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_io with the gate schema | 79 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_lock | 12 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_lock with the gate schema | 12 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_progress_analyze | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_progress_analyze with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_progress_basebackup | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_progress_basebackup with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_progress_cluster | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_progress_cluster with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_progress_copy | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_progress_copy with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_progress_create_index | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_progress_create_index with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_progress_repack | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_progress_repack with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_progress_vacuum | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_progress_vacuum with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_recovery | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_recovery with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_recovery_prefetch | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_recovery_prefetch with the gate schema | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_replication | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_replication with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_replication_slots | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_replication_slots with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_slru | 8 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_slru with the gate schema | 8 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_ssl | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_ssl with the gate schema | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_subscription | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_subscription with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_subscription_stats | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_subscription_stats with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_user_functions | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_user_functions with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_user_indexes | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_user_indexes with the gate schema | 10 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_user_tables | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_user_tables with the gate schema | 5 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_wal | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_wal with the gate schema | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_wal_receiver | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_wal_receiver with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_xact_all_tables | 107 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_xact_all_tables with the gate schema | 116 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_xact_sys_tables | 107 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_xact_sys_tables with the gate schema | 111 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_xact_user_functions | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_xact_user_functions with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_xact_user_tables | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stat_xact_user_tables with the gate schema | 5 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_all_indexes | 163 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_all_indexes with the gate schema | 177 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_all_sequences | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_all_sequences with the gate schema | 3 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_all_tables | 107 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_all_tables with the gate schema | 116 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_sys_indexes | 163 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_sys_indexes with the gate schema | 167 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_sys_sequences | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_sys_sequences with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_sys_tables | 107 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_sys_tables with the gate schema | 111 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_user_indexes | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_user_indexes with the gate schema | 10 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_user_sequences | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_user_sequences with the gate schema | 3 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_user_tables | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_statio_user_tables with the gate schema | 5 | no result |
| L2 | 19 rows of pg_catalog.pg_statistic | 417 | no result |
| L2 | 19 rows of pg_catalog.pg_statistic with the gate schema | 417 | no result |
| L2 | 19 rows of pg_catalog.pg_statistic_ext | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_statistic_ext with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_statistic_ext_data | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_statistic_ext_data with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stats | 417 | no result |
| L2 | 19 rows of pg_catalog.pg_stats with the gate schema | 417 | no result |
| L2 | 19 rows of pg_catalog.pg_stats_ext | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stats_ext with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stats_ext_exprs | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_stats_ext_exprs with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_subscription | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_subscription with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_subscription_rel | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_subscription_rel with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_tables | 68 | no result |
| L2 | 19 rows of pg_catalog.pg_tables with the gate schema | 73 | no result |
| L2 | 19 rows of pg_catalog.pg_tablespace | 2 | no result |
| L2 | 19 rows of pg_catalog.pg_tablespace with the gate schema | 2 | no result |
| L2 | 19 rows of pg_catalog.pg_timezone_abbrevs | 195 | no result |
| L2 | 19 rows of pg_catalog.pg_timezone_abbrevs with the gate schema | 195 | no result |
| L2 | 19 rows of pg_catalog.pg_timezone_names | 598 | no result |
| L2 | 19 rows of pg_catalog.pg_timezone_names with the gate schema | 598 | no result |
| L2 | 19 rows of pg_catalog.pg_transform | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_transform with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_trigger | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_trigger with the gate schema | 12 | no result |
| L2 | 19 rows of pg_catalog.pg_ts_config | 32 | no result |
| L2 | 19 rows of pg_catalog.pg_ts_config with the gate schema | 32 | no result |
| L2 | 19 rows of pg_catalog.pg_ts_config_map | 608 | no result |
| L2 | 19 rows of pg_catalog.pg_ts_config_map with the gate schema | 608 | no result |
| L2 | 19 rows of pg_catalog.pg_ts_dict | 32 | no result |
| L2 | 19 rows of pg_catalog.pg_ts_dict with the gate schema | 32 | no result |
| L2 | 19 rows of pg_catalog.pg_ts_parser | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_ts_parser with the gate schema | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_ts_template | 5 | no result |
| L2 | 19 rows of pg_catalog.pg_ts_template with the gate schema | 5 | no result |
| L2 | 19 rows of pg_catalog.pg_type | 637 | no result |
| L2 | 19 rows of pg_catalog.pg_type with the gate schema | 649 | no result |
| L2 | 19 rows of pg_catalog.pg_user | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_user with the gate schema | 1 | no result |
| L2 | 19 rows of pg_catalog.pg_user_mapping | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_user_mapping with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_user_mappings | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_user_mappings with the gate schema | 0 | no result |
| L2 | 19 rows of pg_catalog.pg_views | 151 | no result |
| L2 | 19 rows of pg_catalog.pg_views with the gate schema | 152 | no result |
| L2 | 19 rows of pg_catalog.pg_wait_events | 287 | no result |
| L2 | 19 rows of pg_catalog.pg_wait_events with the gate schema | 287 | no result |
| L3 | 19 regression tests | 64 | no result |
| L3 | 19 type input and output cases | 593 | no result |
| L4 | 19 isolation specs | 19 | no result |
| L4 | 19 parameter cases | 1303 | no result |
