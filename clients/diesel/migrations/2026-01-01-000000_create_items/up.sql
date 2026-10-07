CREATE TABLE compat_items (id int4 PRIMARY KEY, name text NOT NULL, price numeric(10, 2), added timestamptz NOT NULL DEFAULT now(), active bool NOT NULL DEFAULT true);
