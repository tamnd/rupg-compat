// The trace scenario of spec/21 section 21.4.2, with Drizzle ORM on node-postgres.
// The schema comes from `drizzle-kit push` in run.sh. The scenario writes and reads rows in transactions, causes an error and leaves the tables for the second `drizzle-kit push` and `drizzle-kit pull`.
import { count, eq } from "drizzle-orm";
import { drizzle } from "drizzle-orm/node-postgres";
import pg from "pg";
import * as schema from "./schema.js";

const pool = new pg.Pool({ max: 1 });
const db = drizzle(pool, { schema });
const { items, orders } = schema;

// 1. Rows in a transaction that commits.
await db.transaction(async (tx) => {
  const [one, two] = await tx
    .insert(items)
    .values([
      { name: "one", price: "1.50", mood: "happy", added: new Date("2026-01-02T03:04:05Z"), extra: { n: 1 } },
      { name: "two", price: "2.50" },
    ])
    .returning();
  await tx.insert(orders).values([{ itemId: one.id, qty: 2 }, { itemId: two.id }]);
});
// 2. A transaction that rolls back.
await db
  .transaction(async (tx) => {
    await tx.insert(items).values({ name: "three", price: "3.50" });
    tx.rollback();
  })
  .catch((e) => {
    if (e.constructor.name !== "TransactionRollbackError") throw e;
  });
// 3. Reads with a relation, a filter and a count.
await db.query.items.findMany({ with: { orders: true }, orderBy: items.id });
await db.select().from(items).where(eq(items.name, "two"));
await db.select({ n: count() }).from(orders).where(eq(orders.qty, 1));
// 4. An update and a prepared statement, run three times.
await db.update(items).set({ price: "5.00" }).where(eq(items.name, "two"));
const byName = db.select().from(items).where(eq(items.name, "one")).prepare("compat_by_name");
for (let i = 0; i < 3; i++) await byName.execute();
// 5. An error: a duplicate name.
try {
  await db.insert(items).values({ name: "one" });
  throw new Error("want a unique violation");
} catch (e) {
  if ((e.cause ?? e).code !== "23505") throw e;
}
await pool.end();
