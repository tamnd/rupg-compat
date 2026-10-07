// The tables of the Drizzle scenario.
import { relations } from "drizzle-orm";
import { index, integer, jsonb, numeric, pgEnum, pgTable, serial, text, timestamp } from "drizzle-orm/pg-core";

export const mood = pgEnum("compat_mood", ["sad", "ok", "happy"]);

export const items = pgTable("compat_items", {
  id: serial("id").primaryKey(),
  name: text("name").notNull().unique(),
  price: numeric("price", { precision: 10, scale: 2 }),
  mood: mood("mood").default("ok"),
  added: timestamp("added", { withTimezone: true }),
  extra: jsonb("extra"),
});

export const orders = pgTable(
  "compat_orders",
  {
    id: integer("id").primaryKey().generatedAlwaysAsIdentity(),
    itemId: integer("item_id").notNull().references(() => items.id, { onDelete: "cascade" }),
    qty: integer("qty").notNull().default(1),
  },
  (t) => [index("compat_orders_item").on(t.itemId)],
);

export const itemsRelations = relations(items, ({ many }) => ({ orders: many(orders) }));
export const ordersRelations = relations(orders, ({ one }) => ({ item: one(items, { fields: [orders.itemId], references: [items.id] }) }));
