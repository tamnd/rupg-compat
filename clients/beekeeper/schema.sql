-- The schema that the Beekeeper Studio scenario reads.
CREATE SCHEMA compat_beekeeper;
CREATE TABLE compat_beekeeper.compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2) NOT NULL, created_at timestamptz NOT NULL);
CREATE SEQUENCE compat_beekeeper.compat_orders_code_seq START 1000;
CREATE TABLE compat_beekeeper.compat_orders (id int4 PRIMARY KEY, code int8 NOT NULL DEFAULT nextval('compat_beekeeper.compat_orders_code_seq'), item_id int4 NOT NULL REFERENCES compat_beekeeper.compat_items ON DELETE CASCADE, qty int4 NOT NULL CONSTRAINT compat_orders_qty_check CHECK (qty > 0));
CREATE UNIQUE INDEX compat_orders_code_key ON compat_beekeeper.compat_orders (code);
CREATE VIEW compat_beekeeper.compat_item_totals AS SELECT i.id, i.name, sum(o.qty) AS qty FROM compat_beekeeper.compat_items AS i JOIN compat_beekeeper.compat_orders AS o ON o.item_id = i.id GROUP BY i.id, i.name;
CREATE FUNCTION compat_beekeeper.compat_item_qty(p_item int4) RETURNS int8 LANGUAGE sql STABLE AS $$ SELECT sum(qty) FROM compat_beekeeper.compat_orders WHERE item_id = p_item $$;
COMMENT ON TABLE compat_beekeeper.compat_orders IS 'The orders of the Beekeeper Studio scenario.';
INSERT INTO compat_beekeeper.compat_items SELECT i, 'item ' || i, i * 1.5, '2026-01-02 03:00:00+00'::timestamptz + i * interval '1 hour' FROM generate_series(1, 30) AS i;
INSERT INTO compat_beekeeper.compat_orders (id, item_id, qty) SELECT i, i % 30 + 1, i % 4 + 1 FROM generate_series(1, 60) AS i;
