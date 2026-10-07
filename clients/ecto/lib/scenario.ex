defmodule Scenario.Repo do
  use Ecto.Repo, otp_app: :scenario, adapter: Ecto.Adapters.Postgres
end

defmodule Scenario.Item do
  use Ecto.Schema
  import Ecto.Changeset

  schema "compat_items" do
    field :name, :string
    field :price, :decimal
    field :mood, Ecto.Enum, values: [:happy, :sad]
    field :tags, {:array, :string}
    field :extra, :map
    has_many :orders, Scenario.Order, foreign_key: :item_id
    timestamps(type: :utc_datetime_usec)
  end

  def changeset(item, attrs) do
    item
    |> cast(attrs, [:name, :price, :mood, :tags, :extra, :inserted_at, :updated_at])
    |> validate_required([:name])
    |> unique_constraint(:name)
  end
end

defmodule Scenario.Order do
  use Ecto.Schema

  schema "compat_orders" do
    field :qty, :integer, default: 1
    belongs_to :item, Scenario.Item
    timestamps(type: :utc_datetime_usec)
  end
end
