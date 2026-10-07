// The trace scenario of spec/21 section 21.4.2, with Prisma Client on the node-postgres adapter.
// The schema comes from `prisma migrate deploy` in run.sh. The scenario writes and reads rows in transactions, causes an error and leaves the tables for `prisma migrate status` and `prisma db pull`.
import { PrismaPg } from "@prisma/adapter-pg";
import { Prisma, PrismaClient } from "@prisma/client";

const prisma = new PrismaClient({ adapter: new PrismaPg({ max: 1 }) });

// 1. Rows in a transaction that commits.
await prisma.$transaction(async (tx) => {
  await tx.item.create({
    data: {
      name: "one",
      price: "1.50",
      mood: "happy",
      added: new Date("2026-01-02T03:04:05Z"),
      tags: ["a", "b"],
      extra: { n: 1 },
      orders: { create: [{ qty: 2 }] },
    },
  });
  await tx.item.create({ data: { name: "two", price: "2.50", orders: { create: [{}] } } });
});
// 2. A transaction that rolls back.
await prisma
  .$transaction(async (tx) => {
    await tx.item.create({ data: { name: "three", price: "3.50" } });
    throw new Error("roll back");
  })
  .catch((e) => {
    if (e.message !== "roll back") throw e;
  });
// 3. Reads with a relation, a filter, a count and an aggregate.
await prisma.item.findMany({ include: { orders: true }, orderBy: { id: "asc" } });
await prisma.item.findUnique({ where: { name: "two" } });
await prisma.order.count({ where: { qty: 1 } });
await prisma.item.aggregate({ _sum: { price: true }, _max: { added: true } });
// 4. An update, an upsert and a raw query with a parameter.
await prisma.item.update({ where: { name: "two" }, data: { price: "5.00" } });
await prisma.item.upsert({ where: { name: "one" }, update: { mood: "ok" }, create: { name: "one" } });
await prisma.$queryRaw`SELECT id, name FROM compat_items WHERE price > ${1}`;
// 5. An error: a duplicate name.
try {
  await prisma.item.create({ data: { name: "one" } });
  throw new Error("want a unique violation");
} catch (e) {
  if (!(e instanceof Prisma.PrismaClientKnownRequestError) || e.code !== "P2002") throw e;
}
await prisma.$disconnect();
