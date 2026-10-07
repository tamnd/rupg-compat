// The Drizzle Kit configuration. The connection comes from the PG environment that `rupg-compat record` sets.
import { defineConfig } from "drizzle-kit";

const env = process.env;
export default defineConfig({
  dialect: "postgresql",
  schema: "./schema.js",
  out: "./drizzle",
  dbCredentials: {
    host: env.PGHOST,
    port: Number(env.PGPORT),
    user: env.PGUSER,
    password: env.PGPASSWORD,
    database: env.PGDATABASE,
    ssl: false,
  },
  tablesFilter: ["compat_*"],
});
