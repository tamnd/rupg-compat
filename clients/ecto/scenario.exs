# The trace scenario of spec/21 section 21.4.2, with Ecto on Postgrex.
# An ORM does not send the same statements as a driver. The scenario does the work of an application: it runs the migrations in priv/repo/migrations as `mix ecto.migrate` does, writes and reads rows in transactions, causes errors and reverts the migrations as `mix ecto.rollback --all` does.
import Ecto.Query
alias Scenario.{Item, Order, Repo}

env = System.get_env()

Application.put_env(:scenario, Repo,
  hostname: env["PGHOST"],
  port: String.to_integer(env["PGPORT"]),
  username: env["PGUSER"],
  password: env["PGPASSWORD"],
  database: env["PGDATABASE"],
  ssl: false,
  pool_size: 2,
  log: false
)

{:ok, _} = Repo.start_link()
migrations = Application.app_dir(:scenario, "priv/repo/migrations")
# 1. Run the migrations.
Ecto.Migrator.run(Repo, migrations, :up, all: true, log: false)
# 2. Rows in a transaction that commits. The times are fixed, so the trace does not depend on the clock.
at = ~U[2026-01-02 03:04:05.000000Z]

{:ok, _} =
  Repo.transaction(fn ->
    one = Repo.insert!(Item.changeset(%Item{}, %{name: "one", price: "1.50", tags: ["a"], extra: %{"n" => 1}, inserted_at: at, updated_at: at}))
    two = Repo.insert!(Item.changeset(%Item{}, %{name: "two", price: "2.50", mood: :sad, tags: ["b", "c"], extra: %{"n" => 2}, inserted_at: at, updated_at: at}))
    Repo.insert!(%Order{item_id: one.id, qty: 2, inserted_at: at, updated_at: at})
    Repo.insert!(%Order{item_id: two.id, inserted_at: at, updated_at: at})
  end)

# 3. A transaction that rolls back.
{:error, :rollback} =
  Repo.transaction(fn ->
    Repo.insert!(Item.changeset(%Item{}, %{name: "three", price: "3.50", inserted_at: at, updated_at: at}))
    Repo.rollback(:rollback)
  end)

# 4. Reads with a preload, a filter, a join and aggregates.
Repo.all(from i in Item, order_by: i.id, preload: :orders)
Repo.get_by!(Item, name: "two")
Repo.aggregate(from(i in Item, where: i.mood == :sad), :count)
Repo.aggregate(from(o in Order, where: o.qty == 1), :count)
Repo.all(from i in Item, join: o in assoc(i, :orders), group_by: i.name, order_by: i.name, select: {i.name, sum(o.qty)})
Repo.query!("SELECT * FROM compat_items WHERE price > $1 ORDER BY id", [2])
# 5. Updates.
Repo.get_by!(Item, name: "two") |> Ecto.Changeset.change(price: Decimal.new("5.00"), updated_at: at) |> Repo.update!()
Repo.update_all(from(o in Order, where: o.qty == 1), set: [qty: 3])
# 6. A unique violation and a division by zero.
{:error, %Ecto.Changeset{errors: [name: _]}} = Repo.insert(Item.changeset(%Item{}, %{name: "one", inserted_at: at, updated_at: at}))
{:error, %Postgrex.Error{postgres: %{code: :division_by_zero}}} = Repo.query("SELECT 1 / 0")
# 7. The end: revert the migrations and drop the table of Ecto.
Ecto.Migrator.run(Repo, migrations, :down, all: true, log: false)
Repo.query!("DROP TABLE schema_migrations")
Repo.stop()
