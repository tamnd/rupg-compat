class CreateItems < ActiveRecord::Migration[8.1]
  def change
    create_enum :compat_mood, ["happy", "sad"]
    create_table :compat_items do |t|
      t.string :name, null: false, limit: 100
      t.decimal :price, precision: 10, scale: 2
      t.enum :mood, enum_type: :compat_mood, default: "happy", null: false
      t.string :tags, array: true
      t.jsonb :extra
      t.timestamps
    end
    add_index :compat_items, :name, unique: true
  end
end
