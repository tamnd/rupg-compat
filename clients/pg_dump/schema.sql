-- The schema that the pg_dump scenario dumps and restores.
CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2) CHECK (price >= 0));
CREATE INDEX compat_items_name ON compat_items (name);
CREATE TABLE compat_orders (id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY, item int4 REFERENCES compat_items, qty int4 DEFAULT 1);
CREATE VIEW compat_totals AS SELECT i.name, sum(o.qty) AS qty FROM compat_items i JOIN compat_orders o ON o.item = i.id GROUP BY i.name;
CREATE FUNCTION compat_double(int4) RETURNS int4 LANGUAGE sql IMMUTABLE AS 'SELECT $1 * 2';
CREATE TYPE compat_mood AS ENUM ('sad', 'ok', 'happy');
COMMENT ON TABLE compat_items IS 'The items of the scenario';
INSERT INTO compat_items VALUES (1, 'one', 1.50), (2, 'two', 2.50), (3, 'three', 3.50);
INSERT INTO compat_orders (item, qty) VALUES (1, 2), (2, 1), (2, 5);
