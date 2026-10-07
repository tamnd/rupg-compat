-- The trace scenario of spec/21 section 21.4.2, in psql. Each client runs the same steps.
-- 1. A simple query.
SELECT 1;
-- 2. A query with parameters, with the extended protocol.
SELECT $1::int4 + 1, $2::text \bind 41 abc \g
-- 3. The common types.
SELECT 42::int2, 42::int4, 9007199254740993::int8, 1.5::float8, 12.345::numeric, true, 'x'::text, '\x0102'::bytea, '2026-01-02 03:04:05+00'::timestamptz, '2026-01-02'::date, ARRAY[1, 2, 3]::int4[], '{"a": 1}'::jsonb, '7a3c8f4e-0d1b-4c2a-9e5f-6b7a8c9d0e1f'::uuid, NULL::text;
-- 4. A table.
CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2));
-- 5. A transaction that commits.
BEGIN;
INSERT INTO compat_items VALUES ($1, $2, $3) \bind 1 one 1.50 \g
INSERT INTO compat_items VALUES ($1, $2, $3) \bind 2 two 2.50 \g
INSERT INTO compat_items VALUES ($1, $2, $3) \bind 3 three 3.50 \g
COMMIT;
-- 6. A transaction that rolls back.
BEGIN;
INSERT INTO compat_items VALUES ($1, $2, $3) \bind 4 four 4.50 \g
ROLLBACK;
-- 7. A named statement, run six times.
SELECT id, name, price FROM compat_items WHERE id >= $1 ORDER BY id \parse compat_select
\bind_named compat_select 1 \g
\bind_named compat_select 1 \g
\bind_named compat_select 1 \g
\bind_named compat_select 1 \g
\bind_named compat_select 1 \g
\bind_named compat_select 1 \g
\close_prepared compat_select
-- 8. An update.
UPDATE compat_items SET price = price * 2 WHERE id = $1 \bind 2 \g
-- 9. An error, then an error in a transaction.
SELECT 1 / 0;
BEGIN;
SELECT 1 / 0;
ROLLBACK;
-- 10. The end.
DROP TABLE compat_items;
