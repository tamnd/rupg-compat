defmodule Scenario.Repo.Migrations.CreateItems do
  use Ecto.Migration

  def change do
    execute "CREATE TYPE compat_mood AS ENUM ('happy', 'sad')", "DROP TYPE compat_mood"

    create table(:compat_items) do
      add :name, :string, size: 100, null: false
      add :price, :decimal, precision: 10, scale: 2
      add :mood, :compat_mood, null: false, default: "happy"
      add :tags, {:array, :text}
      add :extra, :map
      timestamps(type: :utc_datetime_usec)
    end

    create unique_index(:compat_items, [:name])
  end
end
