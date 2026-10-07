// The EF Core scenario. `up` applies the migration and uses the tables. `down` reverts the migration and drops the history table and the enum type. The Down method of the migration does not drop the type.
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Infrastructure;
using Microsoft.EntityFrameworkCore.Migrations;
using Npgsql;

var added = new DateTime(2026, 1, 2, 3, 4, 5, DateTimeKind.Utc);
if (args is ["up"])
{
    using (var db = new CompatContext())
    {
        db.Database.Migrate();
    }
    // Add rows in one SaveChanges, which is one transaction.
    using (var db = new CompatContext())
    {
        var one = new Item { Name = "one", Price = 1.5m, Mood = Mood.Happy };
        var two = new Item { Name = "two", Price = 2.5m, Mood = Mood.Sad };
        db.Items.AddRange(one, two);
        db.Orders.Add(new Order { Item = one, Quantity = 3, Added = added });
        db.Orders.Add(new Order { Item = two, Quantity = 1, Added = added });
        db.SaveChanges();
    }
    // Read the rows with LINQ, change one and use an explicit transaction that rolls back.
    using (var db = new CompatContext())
    {
        var items = db.Items.Include(i => i.Orders).Where(i => i.Price > 1m).OrderBy(i => i.Name).ToList();
        var happy = db.Items.Where(i => i.Mood == Mood.Happy).Select(i => new { i.Name, Count = i.Orders.Count }).ToList();
        var total = db.Orders.Sum(o => o.Quantity);
        items[0].Price = 3m;
        db.SaveChanges();
        using (var tx = db.Database.BeginTransaction())
        {
            db.Items.Add(new Item { Name = "three", Price = 3.5m, Mood = Mood.Happy });
            db.SaveChanges();
            tx.Rollback();
        }
        db.Items.Where(i => i.Name == "two").ExecuteUpdate(s => s.SetProperty(i => i.Price, i => i.Price * 2));
        var raw = db.Items.FromSql($"SELECT * FROM compat_items WHERE \"Price\" >= {1m}").ToList();
    }
    // A unique key error.
    using (var db = new CompatContext())
    {
        db.Items.Add(new Item { Name = "one", Price = 1m, Mood = Mood.Sad });
        try
        {
            db.SaveChanges();
            throw new InvalidOperationException("want a unique violation");
        }
        catch (DbUpdateException e) when (e.InnerException is PostgresException { SqlState: "23505" })
        {
        }
    }
}
else if (args is ["down"])
{
    using var db = new CompatContext();
    db.GetService<IMigrator>().Migrate("0");
    db.Database.ExecuteSqlRaw("DROP TABLE \"__EFMigrationsHistory\"");
    db.Database.ExecuteSqlRaw("DROP TYPE compat_mood");
}
else
{
    throw new ArgumentException("use up or down");
}
