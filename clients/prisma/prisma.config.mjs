// The Prisma configuration. The connection comes from the PG environment that `rupg-compat record` sets.
import { defineConfig } from "prisma/config";

const env = process.env;
const url = `postgresql://${env.PGUSER}:${env.PGPASSWORD}@${env.PGHOST}:${env.PGPORT}/${env.PGDATABASE}?sslmode=disable`;
export default defineConfig({ schema: "prisma/schema.prisma", datasource: { url } });
