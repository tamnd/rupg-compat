-- The tables that the Metabase scenario reads.
CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL UNIQUE, price numeric(10, 2), mood text NOT NULL, created_at timestamptz NOT NULL);
CREATE TABLE compat_orders (id int4 PRIMARY KEY, item_id int4 NOT NULL REFERENCES compat_items, qty int4 NOT NULL);
INSERT INTO compat_items SELECT i, 'item ' || i, i * 1.5, CASE WHEN i % 3 = 0 THEN 'sad' ELSE 'happy' END, '2026-01-02 03:00:00+00'::timestamptz + i * interval '1 hour' FROM generate_series(1, 30) AS i;
INSERT INTO compat_orders SELECT i, i % 30 + 1, i % 4 + 1 FROM generate_series(1, 60) AS i;
