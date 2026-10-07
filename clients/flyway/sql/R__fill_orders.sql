INSERT INTO compat_orders (item_id, qty) SELECT id, 2 FROM compat_items WHERE NOT EXISTS (SELECT FROM compat_orders);
