"""The trace scenario of spec/21 section 21.4.2, with Django on psycopg 3.

An ORM does not send the same statements as a driver. The scenario does the work of an application: it creates a table from a model, writes and reads rows, reads the schema with the introspection of the backend and drops the table.
"""

import os
from decimal import Decimal

import django
from django.conf import settings

settings.configure(
    DATABASES={
        "default": {
            "ENGINE": "django.db.backends.postgresql",
            "NAME": os.environ["PGDATABASE"],
            "USER": os.environ["PGUSER"],
            "PASSWORD": os.environ["PGPASSWORD"],
            "HOST": os.environ["PGHOST"],
            "PORT": os.environ["PGPORT"],
            "OPTIONS": {"sslmode": os.environ.get("PGSSLMODE", "prefer")},
        }
    },
    INSTALLED_APPS=[],
    USE_TZ=True,
)
django.setup()

from django.db import DatabaseError, connection, models, transaction  # noqa: E402
from django.db.models import F  # noqa: E402


class Item(models.Model):
    id = models.IntegerField(primary_key=True)
    name = models.TextField()
    price = models.DecimalField(max_digits=10, decimal_places=2, null=True)

    class Meta:
        app_label = "compat"
        db_table = "compat_items"


# 1. A table from the model.
with connection.schema_editor() as editor:
    editor.create_model(Item)
# 2. A transaction that commits, and one that rolls back.
with transaction.atomic():
    Item.objects.create(id=1, name="one", price=Decimal("1.50"))
    Item.objects.create(id=2, name="two", price=Decimal("2.50"))
try:
    with transaction.atomic():
        Item.objects.create(id=4, name="four", price=Decimal("4.50"))
        raise RuntimeError("roll back")
except RuntimeError:
    pass
# 3. The same query six times, and an update.
for _ in range(6):
    list(Item.objects.filter(id__gte=1).order_by("id"))
Item.objects.filter(id=2).update(price=F("price") * 2)
# 4. The introspection of the backend.
with connection.cursor() as cursor:
    connection.introspection.get_table_list(cursor)
    connection.introspection.get_table_description(cursor, "compat_items")
    connection.introspection.get_constraints(cursor, "compat_items")
    connection.introspection.get_sequences(cursor, "compat_items")
# 5. An error.
try:
    with connection.cursor() as cursor:
        cursor.execute("SELECT 1 / 0")
except DatabaseError as e:
    assert e.__cause__.sqlstate == "22012", e
# 6. The end.
with connection.schema_editor() as editor:
    editor.delete_model(Item)
connection.close()
