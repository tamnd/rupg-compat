// The trace scenario of spec/21 section 21.4.2, with Hibernate ORM on pgjdbc.
// An ORM does not send the same statements as a driver. The scenario does the work of an application: Hibernate makes the schema from the entities, the scenario writes and reads rows in transactions and causes an error, a second session factory validates the schema against the catalogs, and the schema manager drops it.
import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.ManyToOne;
import jakarta.persistence.OneToMany;
import jakarta.persistence.Table;
import java.math.BigDecimal;
import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.List;
import org.hibernate.SessionFactory;
import org.hibernate.cfg.Configuration;
import org.hibernate.exception.DataException;

public class Scenario {
    @Entity
    @Table(name = "compat_items")
    public static class Item {
        @Id
        @GeneratedValue(strategy = GenerationType.IDENTITY)
        Integer id;

        @Column(nullable = false, unique = true)
        String name;

        @Column(precision = 10, scale = 2)
        BigDecimal price;

        OffsetDateTime added;

        @OneToMany(mappedBy = "item")
        List<Order> orders = new ArrayList<>();
    }

    @Entity
    @Table(name = "compat_orders")
    public static class Order {
        @Id
        @GeneratedValue(strategy = GenerationType.IDENTITY)
        Integer id;

        @ManyToOne(optional = false)
        Item item;

        int qty = 1;
    }

    static SessionFactory factory(String action) {
        var env = System.getenv();
        return new Configuration()
                .addAnnotatedClass(Item.class)
                .addAnnotatedClass(Order.class)
                .setProperty("hibernate.connection.url", "jdbc:postgresql://" + env.get("PGHOST") + ":" + env.get("PGPORT") + "/" + env.get("PGDATABASE") + "?sslmode=disable")
                .setProperty("hibernate.connection.username", env.get("PGUSER"))
                .setProperty("hibernate.connection.password", env.get("PGPASSWORD"))
                .setProperty("hibernate.connection.pool_size", "1")
                .setProperty("hibernate.hbm2ddl.auto", action)
                .buildSessionFactory();
    }

    static Item item(String name, String price) {
        var i = new Item();
        i.name = name;
        i.price = new BigDecimal(price);
        return i;
    }

    public static void main(String[] args) {
        // 1. Hibernate makes the schema.
        try (var sf = factory("create")) {
            // 2. Rows in a transaction that commits.
            sf.inTransaction(s -> {
                var one = item("one", "1.50");
                one.added = OffsetDateTime.parse("2026-01-02T03:04:05Z");
                var two = item("two", "2.50");
                s.persist(one);
                s.persist(two);
                var o = new Order();
                o.item = one;
                o.qty = 2;
                s.persist(o);
                var p = new Order();
                p.item = two;
                s.persist(p);
            });
            // 3. A transaction that rolls back.
            var s = sf.openSession();
            var t = s.beginTransaction();
            s.persist(item("three", "3.50"));
            s.flush();
            t.rollback();
            s.close();
            // 4. Reads with a join, a filter and a count.
            sf.inSession(r -> {
                r.createSelectionQuery("from Scenario$Item i left join fetch i.orders order by i.id", Item.class).getResultList();
                r.createSelectionQuery("from Scenario$Item where name = :name", Item.class).setParameter("name", "two").getSingleResult();
                r.createSelectionQuery("select count(*) from Scenario$Order where qty = 1", Long.class).getSingleResult();
            });
            // 5. An update.
            sf.inTransaction(u -> u.createMutationQuery("update Scenario$Item set price = :p where name = :n").setParameter("p", new BigDecimal("5.00")).setParameter("n", "two").executeUpdate());
            // 6. An error.
            try {
                sf.inSession(e -> e.createNativeQuery("SELECT 1 / 0", Integer.class).getSingleResult());
                throw new IllegalStateException("want division by zero");
            } catch (DataException e) {
                if (!"22012".equals(e.getSQLState())) {
                    throw e;
                }
            }
        }
        // 7. A second factory validates the schema, and the schema manager drops it.
        try (var sf = factory("validate")) {
            sf.getSchemaManager().dropMappedObjects(true);
        }
    }
}
