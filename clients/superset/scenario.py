"""The trace scenario of spec/21 section 21.4.2, with Superset on psycopg2.

A BI tool does not send the same statements as a driver. The scenario sends the requests of the Superset web pages to the REST API, through the test client of Flask: it tests the connection, adds the database, lists the schemas and the tables, reads the metadata of a table, adds a dataset, runs a chart query and runs two SQL Lab queries, one of which fails.
"""

import json
import os
import sys

from superset.app import create_app

app = create_app()
client = app.test_client()
uri = "postgresql+psycopg2://{PGUSER}:{PGPASSWORD}@{PGHOST}:{PGPORT}/{PGDATABASE}?sslmode=disable".format(**os.environ)


def call(method, path, body=None, want=200):
    r = client.open(path, method=method, json=body, headers=headers)
    if r.status_code != want:
        sys.exit(f"{method} {path}: {r.status_code} {r.get_data(as_text=True)}")
    return r.get_json()


headers = {}
token = call("POST", "/api/v1/security/login", {"username": "admin", "password": "compat", "provider": "db", "refresh": False})
headers = {"Authorization": f"Bearer {token['access_token']}"}

# 1. Test the connection, from the page that adds a database.
call("POST", "/api/v1/database/test_connection/", {"database_name": "compat", "sqlalchemy_uri": uri})

# 2. Add the database.
db = call("POST", "/api/v1/database/", {"database_name": "compat", "sqlalchemy_uri": uri, "expose_in_sqllab": True}, want=201)["id"]

# 3. The schemas and the tables, from the dataset page and SQL Lab.
schemas = call("GET", f"/api/v1/database/{db}/schemas/")["result"]
if "public" not in schemas:
    sys.exit(f"no schema public in {schemas}")
tables = call("GET", f"/api/v1/database/{db}/tables/?q=(schema_name:public,force:!t)")["result"]
if {t["value"] for t in tables} != {"compat_regions", "compat_sales"}:
    sys.exit(f"tables: {tables}")

# 4. The metadata of a table, which SQLAlchemy reflects.
meta = call("GET", f"/api/v1/database/{db}/table_metadata/?name=compat_sales&schema=public")
call("GET", f"/api/v1/database/{db}/table_metadata/extra/?name=compat_sales&schema=public")

# 5. A dataset on the table.
dataset = call("POST", "/api/v1/dataset/", {"database": db, "schema": "public", "table_name": "compat_sales"}, want=201)["id"]

# 6. A chart query: the sum of the amounts for each region.
context = {
    "datasource": {"id": dataset, "type": "table"},
    "queries": [{"columns": ["region"], "metrics": [{"expressionType": "SQL", "sqlExpression": "SUM(amount)", "label": "total"}], "orderby": [["region", True]], "row_limit": 100}],
    "result_format": "json",
    "result_type": "full",
}
result = call("POST", "/api/v1/chart/data", context)["result"][0]
if result["rowcount"] != 2:
    sys.exit(f"chart: {json.dumps(result)[:500]}")

# 7. Two SQL Lab queries. The second fails with a division by zero.
query = {"database_id": db, "schema": "public", "runAsync": False, "sql": "SELECT s.sold, r.name, s.amount FROM compat_sales s JOIN compat_regions r ON r.id = s.region ORDER BY s.id"}
rows = call("POST", "/api/v1/sqllab/execute/", query)
if len(rows["data"]) != 20:
    sys.exit(f"sqllab: {json.dumps(rows)[:500]}")
r = client.post("/api/v1/sqllab/execute/", json=dict(query, sql="SELECT 1 / 0"), headers=headers)
if "division by zero" not in r.get_data(as_text=True):
    sys.exit(f"want division by zero, got {r.status_code} {r.get_data(as_text=True)[:500]}")
