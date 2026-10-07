defmodule Scenario.Repo.Migrations.CreateOrders do
  use Ecto.Migration

  def change do
    create table(:compat_orders) do
      add :item_id, references(:compat_items), null: false
      add :qty, :integer, null: false, default: 1
      timestamps(type: :utc_datetime_usec)
    end

    create index(:compat_orders, [:item_id])
  end
end
