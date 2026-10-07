CREATE TABLE compat_orders (
    id int4 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    item_id int4 NOT NULL REFERENCES compat_items (id),
    qty int4 NOT NULL DEFAULT 1
);
CREATE INDEX compat_orders_item ON compat_orders (item_id);
CREATE VIEW compat_totals AS SELECT i.name, sum(o.qty) AS qty FROM compat_items i JOIN compat_orders o ON o.item_id = i.id GROUP BY i.name;
