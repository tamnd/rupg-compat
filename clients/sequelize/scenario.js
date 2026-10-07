// The trace scenario of spec/21 section 21.4.2, with Sequelize on node-postgres.
// An ORM does not send the same statements as a driver. The scenario does the work of an application: it creates the tables from the models, writes and reads rows in transactions, reads the schema with the query interface, causes an error and drops the tables.
import { DataTypes, Sequelize } from "sequelize";

const env = process.env;
const sequelize = new Sequelize(env.PGDATABASE, env.PGUSER, env.PGPASSWORD, {
  host: env.PGHOST,
  port: Number(env.PGPORT),
  dialect: "postgres",
  logging: false,
  pool: { max: 1, min: 0 },
});

const Item = sequelize.define("Item", {
  name: { type: DataTypes.TEXT, allowNull: false },
  price: { type: DataTypes.DECIMAL(10, 2) },
  tags: { type: DataTypes.ARRAY(DataTypes.TEXT) },
  extra: { type: DataTypes.JSONB },
}, { tableName: "compat_items" });
const Order = sequelize.define("Order", {
  qty: { type: DataTypes.INTEGER, allowNull: false, defaultValue: 1 },
}, { tableName: "compat_orders" });
Item.hasMany(Order, { foreignKey: { name: "itemId", allowNull: false } });
Order.belongsTo(Item, { foreignKey: { name: "itemId", allowNull: false } });

// 1. Connect and create the tables from the models.
await sequelize.authenticate();
await sequelize.sync();
// 2. Rows in a transaction that commits.
await sequelize.transaction(async (t) => {
  const items = await Item.bulkCreate(
    [
      { name: "one", price: "1.50", tags: ["a"], extra: { n: 1 } },
      { name: "two", price: "2.50", tags: ["b", "c"], extra: { n: 2 } },
    ],
    { transaction: t, returning: true },
  );
  await Order.create({ itemId: items[0].id, qty: 2 }, { transaction: t });
  await Order.create({ itemId: items[1].id }, { transaction: t });
});
// 3. A transaction that rolls back.
const t = await sequelize.transaction();
await Item.create({ name: "three", price: "3.50" }, { transaction: t });
await t.rollback();
// 4. Reads with a join, a filter and a count.
await Item.findAll({ include: Order, order: [["id", "ASC"]] });
await Item.findOne({ where: { name: "two" } });
await Order.count({ where: { qty: 1 } });
// 5. An update.
await Item.update({ price: "5.00" }, { where: { name: "two" } });
// 6. The schema, as the query interface reads it.
const qi = sequelize.getQueryInterface();
await qi.showAllTables();
await qi.describeTable("compat_items");
await qi.showIndex("compat_orders");
await qi.getForeignKeyReferencesForTable("compat_orders");
// 7. An error.
try {
  await sequelize.query("SELECT 1 / 0");
  throw new Error("want division by zero");
} catch (e) {
  if (e.original?.code !== "22012") throw e;
}
// 8. The end.
await sequelize.drop();
await sequelize.close();
