-- The schema of the pg_dump round trip (spec/21 section 21.4.1). `rupg-compat dump` runs this script on the server under test in a new database, dumps the database with pg_dump of the oracle build, restores the dump into the oracle and dumps the oracle again. The script has only the statements that rupg runs now. Add a statement when rupg can run it.
CREATE SCHEMA shop;
CREATE TABLE shop.items (id serial PRIMARY KEY, name text NOT NULL UNIQUE, price numeric(10,2) DEFAULT 0 CHECK (price >= 0), tags text[], made timestamptz DEFAULT now());
CREATE INDEX items_lower_name ON shop.items (lower(name)) WHERE price > 0;
CREATE TABLE shop.orders (id bigserial PRIMARY KEY, item int NOT NULL REFERENCES shop.items ON DELETE CASCADE, qty int DEFAULT 1, note varchar(200), UNIQUE (item, qty));
CREATE INDEX orders_item ON shop.orders USING hash (item);
CREATE VIEW shop.totals AS SELECT i.name, sum(o.qty) AS qty FROM shop.items i JOIN shop.orders o ON o.item = i.id GROUP BY i.name;
CREATE TABLE plain (a int, b "char", c bytea, d jsonb, e uuid, f interval(3), g bit(4), h inet);
CREATE UNIQUE INDEX plain_a_desc ON plain (a DESC NULLS LAST, e);
COMMENT ON TABLE shop.items IS 'The items';
COMMENT ON COLUMN shop.items.price IS 'The price';
COMMENT ON SCHEMA shop IS 'The shop';
COMMENT ON VIEW shop.totals IS 'The quantity of each item';
