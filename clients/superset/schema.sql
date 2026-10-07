-- The tables that the Superset scenario reads.
CREATE TABLE compat_regions (id int4 PRIMARY KEY, name text NOT NULL UNIQUE);
CREATE TABLE compat_sales (id int8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY, region int4 NOT NULL REFERENCES compat_regions, sold date NOT NULL, amount numeric(10, 2) NOT NULL);
CREATE INDEX compat_sales_sold ON compat_sales (sold);
COMMENT ON TABLE compat_sales IS 'The sales of the scenario';
INSERT INTO compat_regions VALUES (1, 'north'), (2, 'south');
INSERT INTO compat_sales (region, sold, amount) SELECT 1 + i % 2, DATE '2026-01-01' + i, 10 + i FROM generate_series(0, 19) AS i;
