CREATE TABLE compat_orders (id int8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY, item int4 NOT NULL REFERENCES compat_items, qty int4 NOT NULL DEFAULT 1);
CREATE INDEX compat_orders_item ON compat_orders (item);
