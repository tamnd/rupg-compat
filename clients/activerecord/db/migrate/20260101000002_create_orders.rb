class CreateOrders < ActiveRecord::Migration[8.1]
  def change
    create_table :compat_orders do |t|
      t.references :item, null: false, foreign_key: { to_table: :compat_items }
      t.integer :qty, null: false, default: 1
      t.timestamps
    end
  end
end
