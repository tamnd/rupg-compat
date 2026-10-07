// The model of the EF Core scenario: an enum type, two tables with a foreign key and an index.
using Microsoft.EntityFrameworkCore;

public enum Mood
{
    Happy,
    Sad,
}

public class Item
{
    public int Id { get; set; }
    public string Name { get; set; } = "";
    public decimal Price { get; set; }
    public Mood Mood { get; set; }
    public List<Order> Orders { get; set; } = [];
}

public class Order
{
    public int Id { get; set; }
    public int ItemId { get; set; }
    public Item Item { get; set; } = null!;
    public int Quantity { get; set; }
    public DateTime Added { get; set; }
}

public class CompatContext : DbContext
{
    public DbSet<Item> Items => Set<Item>();
    public DbSet<Order> Orders => Set<Order>();

    // The connection comes from the environment of the proxy. `dotnet ef migrations add` needs no connection, so the defaults only let it build the model.
    public static string ConnectionString()
    {
        static string Env(string name, string fallback) => Environment.GetEnvironmentVariable(name) ?? fallback;
        return $"Host={Env("PGHOST", "127.0.0.1")};Port={Env("PGPORT", "5432")};Database={Env("PGDATABASE", "compat")};Username={Env("PGUSER", "postgres")};Password={Env("PGPASSWORD", "")};SSL Mode=Disable;Pooling=false";
    }

    protected override void OnConfiguring(DbContextOptionsBuilder options) =>
        options.UseNpgsql(ConnectionString(), o => o.MapEnum<Mood>("compat_mood"));

    protected override void OnModelCreating(ModelBuilder model)
    {
        model.Entity<Item>(e =>
        {
            e.ToTable("compat_items");
            e.Property(i => i.Name).HasMaxLength(100);
            e.Property(i => i.Price).HasPrecision(10, 2);
            e.HasIndex(i => i.Name).IsUnique();
        });
        model.Entity<Order>(e =>
        {
            e.ToTable("compat_orders");
            e.HasOne(o => o.Item).WithMany(i => i.Orders).HasForeignKey(o => o.ItemId);
        });
    }
}
