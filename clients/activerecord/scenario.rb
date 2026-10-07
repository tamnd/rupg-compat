# The trace scenario of spec/21 section 21.4.2, with Active Record, the ORM of Rails, on the pg gem.
# An ORM does not send the same statements as a driver. The scenario does the work of an application: it runs the migrations in db/migrate, writes and reads rows in transactions, dumps the schema as `rails db:schema:dump` does, causes errors and reverts the migrations.
require "active_record"
require "stringio"

ActiveRecord::Base.establish_connection(
  adapter: "postgresql",
  host: ENV.fetch("PGHOST"),
  port: ENV.fetch("PGPORT"),
  database: ENV.fetch("PGDATABASE"),
  username: ENV.fetch("PGUSER"),
  password: ENV.fetch("PGPASSWORD"),
  sslmode: "disable",
  pool: 1,
)

class Item < ActiveRecord::Base
  self.table_name = "compat_items"
  enum :mood, { happy: "happy", sad: "sad" }
  has_many :orders
  validates :name, presence: true
end

class Order < ActiveRecord::Base
  self.table_name = "compat_orders"
  belongs_to :item
end

migrations = ActiveRecord::MigrationContext.new(File.join(__dir__, "db/migrate"))
# 1. Run the migrations, as `rails db:migrate` does.
migrations.migrate
# 2. Rows in a transaction that commits. The times are fixed, so the trace does not depend on the clock.
at = Time.utc(2026, 1, 2, 3, 4, 5)
Item.transaction do
  one = Item.create!(name: "one", price: "1.50", tags: ["a"], extra: { "n" => 1 }, created_at: at, updated_at: at)
  two = Item.create!(name: "two", price: "2.50", mood: :sad, tags: ["b", "c"], extra: { "n" => 2 }, created_at: at, updated_at: at)
  one.orders.create!(qty: 2, created_at: at, updated_at: at)
  two.orders.create!(created_at: at, updated_at: at)
end
# 3. A transaction that rolls back.
Item.transaction do
  Item.create!(name: "three", price: "3.50", created_at: at, updated_at: at)
  raise ActiveRecord::Rollback
end
# 4. Reads with eager loading, a filter, a join and aggregates.
Item.includes(:orders).order(:id).to_a
Item.find_by(name: "two")
Item.sad.count
Order.where(qty: 1).count
Item.joins(:orders).group(:name).order(:name).sum(:qty)
Item.find_by_sql(["SELECT * FROM compat_items WHERE price > ? ORDER BY id", 2]).to_a
# 5. Updates.
Item.find_by!(name: "two").update!(price: "5.00", updated_at: at)
Order.where(qty: 1).update_all(qty: 3)
# 6. The schema, as `rails db:schema:dump` reads it.
ActiveRecord::SchemaDumper.dump(ActiveRecord::Base.connection_pool, StringIO.new)
# 7. A unique violation and a division by zero.
begin
  Item.create!(name: "one", created_at: at, updated_at: at)
  raise "want a unique violation"
rescue ActiveRecord::RecordNotUnique
end
begin
  ActiveRecord::Base.connection.select_all("SELECT 1 / 0")
  raise "want division by zero"
rescue ActiveRecord::StatementInvalid => e
  raise unless e.cause.is_a?(PG::DivisionByZero)
end
# 8. The end: revert the migrations, as `rails db:migrate VERSION=0` does, and drop the tables of Rails.
migrations.migrate(0)
ActiveRecord::Base.connection.drop_table(:schema_migrations)
ActiveRecord::Base.connection.drop_table(:ar_internal_metadata)
ActiveRecord::Base.connection_pool.disconnect!
