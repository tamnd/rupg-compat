-- The gate schema of G1 (spec/21 section 21.4.2). `rupg-compat gate` runs this script in a new database on each server. Then each of the seven clients lists the tables, the columns, the types, the indexes and the constraints of the schema gate, and the list on the server under test must be equal to the list on the oracle.
CREATE SCHEMA gate;
COMMENT ON SCHEMA gate IS 'The gate schema of G1';
CREATE TABLE gate.customers (id bigserial PRIMARY KEY, email varchar(320) NOT NULL UNIQUE, name text NOT NULL, born date, active boolean NOT NULL DEFAULT true, created timestamptz NOT NULL DEFAULT now());
COMMENT ON TABLE gate.customers IS 'The customers';
COMMENT ON COLUMN gate.customers.email IS 'The address for mail';
CREATE TABLE gate.products (id serial PRIMARY KEY, sku char(12) NOT NULL, title text NOT NULL, price numeric(12,2) NOT NULL CHECK (price >= 0), weight real, tags text[] DEFAULT '{}', data jsonb, CONSTRAINT products_sku_key UNIQUE (sku), CONSTRAINT products_title_check CHECK (length(title) > 0));
CREATE TABLE gate.orders (id bigserial PRIMARY KEY, customer bigint NOT NULL REFERENCES gate.customers (id) ON DELETE CASCADE, placed timestamp(3) without time zone NOT NULL, note varchar, total numeric(14,2) NOT NULL DEFAULT 0);
CREATE TABLE gate.order_lines (order_id bigint NOT NULL REFERENCES gate.orders ON DELETE CASCADE, line smallint NOT NULL, product int NOT NULL REFERENCES gate.products (id), qty int NOT NULL CHECK (qty > 0), PRIMARY KEY (order_id, line));
CREATE INDEX orders_customer_placed ON gate.orders (customer, placed DESC);
CREATE INDEX products_lower_title ON gate.products (lower(title));
CREATE INDEX orders_open ON gate.orders (placed) WHERE total = 0;
CREATE UNIQUE INDEX customers_lower_email ON gate.customers (lower(email::text));
CREATE TABLE gate.kinds (a int2, b int4, c int8, d float4, e float8, f numeric, g money, h char, i "char", j name, k bytea, l bit(3), m varbit(5), n uuid, o inet, p cidr, q macaddr, r interval, s time, t timetz, u xml, v json, w point, x int4range, y tsvector, z oid);
CREATE VIEW gate.customer_orders AS SELECT c.id, c.email, count(o.id) AS orders, sum(o.total) AS total FROM gate.customers c LEFT JOIN gate.orders o ON o.customer = c.id GROUP BY c.id, c.email;
COMMENT ON VIEW gate.customer_orders IS 'The orders of each customer';
