-- The table that the pgAdmin scenario reads.
CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL UNIQUE, price numeric(10, 2) CHECK (price >= 0), created_at timestamptz NOT NULL DEFAULT '2026-01-02 03:00:00+00');
CREATE INDEX compat_items_price ON compat_items (price);
COMMENT ON TABLE compat_items IS 'The items of the compat scenario';
INSERT INTO compat_items (id, name, price) SELECT i, 'item ' || i, i * 1.5 FROM generate_series(1, 5) AS i;
