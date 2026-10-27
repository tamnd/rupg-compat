-- The schema that the DBeaver scenario reads.
CREATE SCHEMA compat_dbeaver;
CREATE SEQUENCE compat_dbeaver.item_ids;
CREATE TABLE compat_dbeaver.items (id int4 PRIMARY KEY DEFAULT nextval('compat_dbeaver.item_ids'), name text NOT NULL, price numeric(10, 2) NOT NULL CHECK (price >= 0));
CREATE UNIQUE INDEX items_lower_name ON compat_dbeaver.items (lower(name));
CREATE TABLE compat_dbeaver.orders (id int4 PRIMARY KEY, item_id int4 NOT NULL REFERENCES compat_dbeaver.items, qty int4 NOT NULL);
CREATE VIEW compat_dbeaver.item_totals AS SELECT i.id, i.name, coalesce(sum(o.qty), 0) AS qty FROM compat_dbeaver.items i LEFT JOIN compat_dbeaver.orders o ON o.item_id = i.id GROUP BY i.id, i.name;
COMMENT ON TABLE compat_dbeaver.items IS 'The items for sale';
COMMENT ON COLUMN compat_dbeaver.items.name IS 'The name of the item';
-- 250 rows, so that the LIMIT 200 of the data editor has an effect.
INSERT INTO compat_dbeaver.items (name, price) SELECT 'item ' || i, i * 1.5 FROM generate_series(1, 250) AS i;
INSERT INTO compat_dbeaver.orders SELECT i, i % 250 + 1, i % 4 + 1 FROM generate_series(1, 60) AS i;
