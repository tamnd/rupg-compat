"""The trace scenario of spec/21 section 21.4.2, with SQLAlchemy and Alembic on psycopg 3.

An ORM does not send the same statements as a driver. The scenario does the work of an application: it creates the schema from the models, writes and reads rows in sessions, reflects the schema and lets Alembic compare the schema with the models.
"""

from decimal import Decimal

from alembic.autogenerate import compare_metadata
from alembic.migration import MigrationContext
from alembic.operations import Operations
from sqlalchemy import Column, Integer, Numeric, Text, create_engine, inspect, select, text, update
from sqlalchemy.exc import DBAPIError
from sqlalchemy.orm import DeclarativeBase, Session


class Base(DeclarativeBase):
    pass


class Item(Base):
    __tablename__ = "compat_items"
    id = Column(Integer, primary_key=True, autoincrement=False)
    name = Column(Text, nullable=False)
    price = Column(Numeric(10, 2))


# psycopg reads PGHOST, PGPORT, PGUSER, PGPASSWORD, PGDATABASE and PGSSLMODE. One connection, so that the trace has one session.
engine = create_engine("postgresql+psycopg://", pool_size=1, max_overflow=0)
# 1. The schema from the models.
Base.metadata.create_all(engine)
# 2. A session that commits, and one that rolls back.
with Session(engine) as s:
    s.add_all([Item(id=1, name="one", price=Decimal("1.50")), Item(id=2, name="two", price=Decimal("2.50"))])
    s.commit()
with Session(engine) as s:
    s.add(Item(id=4, name="four", price=Decimal("4.50")))
    s.flush()
    s.rollback()
# 3. The same query six times, and an update.
with Session(engine) as s:
    for _ in range(6):
        s.scalars(select(Item).where(Item.id >= 1).order_by(Item.id)).all()
    s.execute(update(Item).where(Item.id == 2).values(price=Item.price * 2))
    s.commit()
# 4. Reflection.
insp = inspect(engine)
insp.get_table_names()
insp.get_columns("compat_items")
insp.get_pk_constraint("compat_items")
insp.get_indexes("compat_items")
insp.get_foreign_keys("compat_items")
# 5. Alembic: compare the schema with the models, then add a column.
with engine.begin() as conn:
    ctx = MigrationContext.configure(conn)
    ctx.get_current_revision()
    compare_metadata(ctx, Base.metadata)
    Operations(ctx).add_column("compat_items", Column("note", Text))
# 6. An error.
with engine.connect() as conn:
    try:
        conn.execute(text("SELECT 1 / 0"))
    except DBAPIError as e:
        assert e.orig.sqlstate == "22012", e
# 7. The end.
Base.metadata.drop_all(engine)
engine.dispose()
