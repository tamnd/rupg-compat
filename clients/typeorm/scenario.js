// The trace scenario of spec/21 section 21.4.2, with TypeORM on node-postgres.
// The rows get fixed times, not now(), so that two runs on the oracle return the same rows and no group is unstable.
// An ORM does not send the same statements as a driver. The scenario does the work of an application: it synchronizes the schema with the entities, writes and reads rows in transactions, reads the schema with a query runner, causes an error and drops the tables.
import "reflect-metadata";
import { DataSource, EntitySchema } from "typeorm";

const env = process.env;
const Item = new EntitySchema({
  name: "Item",
  tableName: "compat_items",
  columns: {
    id: { type: "int", primary: true, generated: true },
    name: { type: "text" },
    price: { type: "numeric", precision: 10, scale: 2, nullable: true },
    added: { type: "timestamptz", nullable: true },
    extra: { type: "jsonb", nullable: true },
  },
  relations: { orders: { type: "one-to-many", target: "Order", inverseSide: "item" } },
});
const Order = new EntitySchema({
  name: "Order",
  tableName: "compat_orders",
  columns: {
    id: { type: "int", primary: true, generated: true },
    qty: { type: "int", default: 1 },
  },
  relations: { item: { type: "many-to-one", target: "Item", joinColumn: true, nullable: false } },
  indices: [{ name: "compat_orders_qty", columns: ["qty"] }],
});

const ds = new DataSource({
  type: "postgres",
  host: env.PGHOST,
  port: Number(env.PGPORT),
  username: env.PGUSER,
  password: env.PGPASSWORD,
  database: env.PGDATABASE,
  entities: [Item, Order],
  synchronize: false,
  logging: false,
  poolSize: 1,
});

// 1. Connect and synchronize the schema with the entities, which reads the catalogs first.
await ds.initialize();
await ds.synchronize();
// 2. Rows in a transaction that commits.
await ds.transaction(async (m) => {
  const one = await m.save(Item, { name: "one", price: "1.50", added: new Date("2026-01-02T03:04:05Z"), extra: { n: 1 } });
  const two = await m.save(Item, { name: "two", price: "2.50" });
  await m.save(Order, [{ item: one, qty: 2 }, { item: two }]);
});
// 3. A transaction that rolls back.
await ds
  .transaction(async (m) => {
    await m.save(Item, { name: "three", price: "3.50" });
    throw new Error("roll back");
  })
  .catch((e) => {
    if (e.message !== "roll back") throw e;
  });
// 4. Reads with a join, a filter and a count.
await ds.getRepository(Item).find({ relations: { orders: true }, order: { id: "ASC" } });
await ds.getRepository(Item).findOneBy({ name: "two" });
await ds.getRepository(Order).countBy({ qty: 1 });
// 5. An update.
await ds.getRepository(Item).update({ name: "two" }, { price: "5.00" });
// 6. The schema, as a query runner reads it. A second synchronize finds no change.
const qr = ds.createQueryRunner();
await qr.getTable("compat_orders");
await qr.release();
await ds.synchronize();
// 7. An error.
try {
  await ds.query("SELECT 1 / 0");
  throw new Error("want division by zero");
} catch (e) {
  if (e.driverError?.code !== "22012") throw e;
}
// 8. The end.
await ds.dropDatabase();
await ds.destroy();
