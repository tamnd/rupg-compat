-- CreateSchema
CREATE SCHEMA IF NOT EXISTS "public";

-- CreateEnum
CREATE TYPE "compat_mood" AS ENUM ('sad', 'ok', 'happy');

-- CreateTable
CREATE TABLE "compat_items" (
    "id" SERIAL NOT NULL,
    "name" TEXT NOT NULL,
    "price" DECIMAL(10,2),
    "mood" "compat_mood" NOT NULL DEFAULT 'ok',
    "added" TIMESTAMPTZ(6),
    "tags" TEXT[],
    "extra" JSONB,

    CONSTRAINT "compat_items_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "compat_orders" (
    "id" SERIAL NOT NULL,
    "item_id" INTEGER NOT NULL,
    "qty" INTEGER NOT NULL DEFAULT 1,

    CONSTRAINT "compat_orders_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE UNIQUE INDEX "compat_items_name_key" ON "compat_items"("name");

-- CreateIndex
CREATE INDEX "compat_orders_item_id_idx" ON "compat_orders"("item_id");

-- AddForeignKey
ALTER TABLE "compat_orders" ADD CONSTRAINT "compat_orders_item_id_fkey" FOREIGN KEY ("item_id") REFERENCES "compat_items"("id") ON DELETE CASCADE ON UPDATE CASCADE;

