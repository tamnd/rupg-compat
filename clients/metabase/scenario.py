"""The trace scenario of spec/21 section 21.4.2, with Metabase.

A BI tool does not send the same statements as a driver. Metabase makes the SQL in its backend from the requests of its web pages. So the scenario sends those requests: the setup of the first user, the form that adds a database, which tests the connection, the sync and analysis of the database, a native query, a native query with a variable, two questions of the query builder and a native query that fails.
"""

import json
import os
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request

out = sys.argv[1]
data = os.path.join(out, "data")

# A free port on the loopback for the HTTP server of Metabase.
with socket.socket() as s:
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]

env = dict(
    os.environ,
    MB_DB_TYPE="h2",
    MB_DB_FILE=os.path.join(data, "metabase"),
    MB_PLUGINS_DIR=os.path.join(data, "plugins"),
    MB_JETTY_HOST="127.0.0.1",
    MB_JETTY_PORT=str(port),
    MB_ANON_TRACKING_ENABLED="false",
    MB_CHECK_FOR_UPDATES="false",
    MB_LOAD_SAMPLE_CONTENT="false",
    MB_SITE_LOCALE="en",
)
log = open(os.path.join(data, "metabase.log"), "w")
server = subprocess.Popen(
    ["java", "-Xmx1g", "-Djava.io.tmpdir=" + data, "-jar", os.path.join(out, "metabase.jar")],
    cwd=data,
    env=env,
    stdout=log,
    stderr=subprocess.STDOUT,
)
session = None


def call(method, path, body=None):
    headers = {"Content-Type": "application/json"}
    if session:
        headers["X-Metabase-Session"] = session
    req = urllib.request.Request(
        f"http://127.0.0.1:{port}{path}",
        method=method,
        data=None if body is None else json.dumps(body).encode(),
        headers=headers,
    )
    try:
        with urllib.request.urlopen(req, timeout=300) as r:
            text = r.read()
            return r.status, json.loads(text) if text else None
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode()


def want(result, what, status=(200, 202)):
    code, body = result
    if code not in status:
        sys.exit(f"{what}: {code} {body}")
    return body


def query(db, q, what):
    """Runs one query, as a question or the native editor does, and checks that it completed."""
    body = want(call("POST", "/api/dataset", dict(q, database=db)), what)
    if body.get("status") != "completed":
        sys.exit(f"{what}: {body.get('status')} {body.get('error')}")
    return body


try:
    for _ in range(600):
        try:
            if call("GET", "/api/health")[0] == 200:
                break
        except OSError:
            pass
        if server.poll() is not None:
            sys.exit("Metabase stopped")
        time.sleep(1)
    else:
        sys.exit("Metabase did not start")

    # 1. The setup of the first user. No database comes with it.
    token = want(call("GET", "/api/session/properties"), "properties")["setup-token"]
    user = {"first_name": "Compat", "last_name": "Compat", "email": "compat@example.com", "password": "compat-Compat-1"}
    body = want(call("POST", "/api/setup", {"token": token, "user": user, "prefs": {"site_name": "compat", "site_locale": "en", "allow_tracking": False}}), "setup")
    session = body["id"]

    # 2. The form that adds a database. Metabase tests the connection, saves the database and starts the sync.
    details = {
        "host": os.environ["PGHOST"],
        "port": int(os.environ["PGPORT"]),
        "dbname": os.environ["PGDATABASE"],
        "user": os.environ["PGUSER"],
        "password": os.environ["PGPASSWORD"],
        "ssl": False,
        "tunnel-enabled": False,
        "advanced-options": False,
    }
    db = want(call("POST", "/api/database", {"engine": "postgres", "name": "compat", "details": details, "is_full_sync": True}), "add database")["id"]

    # 3. Wait until the sync of the database is done: the metadata, the analysis of the fields and the scan of the field values.
    for _ in range(600):
        tasks = want(call("GET", "/api/task/?limit=100&offset=0"), "tasks")
        rows = tasks.get("data", tasks) if isinstance(tasks, dict) else tasks
        if any(t.get("task") == "sync" and t.get("db_id") == db and t.get("ended_at") for t in rows):
            break
        time.sleep(1)
    else:
        sys.exit("the sync did not end")
    meta = want(call("GET", f"/api/database/{db}/metadata"), "metadata")
    tables = {t["name"]: t for t in meta["tables"]}
    items = tables["compat_items"]
    fields = {f["name"]: f["id"] for f in items["fields"]}

    # 4. A native query from the SQL editor.
    query(db, {"type": "native", "native": {"query": "SELECT name, price, mood FROM compat_items WHERE price > 30 ORDER BY id"}}, "native")

    # 5. A native query with a variable.
    tag = {"id": "compat-id", "name": "id", "display-name": "ID", "type": "number"}
    query(
        db,
        {
            "type": "native",
            "native": {"query": "SELECT * FROM compat_items WHERE id = {{id}}", "template-tags": {"id": tag}},
            "parameters": [{"type": "number/=", "target": ["variable", ["template-tag", "id"]], "value": [2]}],
        },
        "native with a variable",
    )

    # 6. Two questions of the query builder: a count by a category, and a filter with a sum and an order.
    query(db, {"type": "query", "query": {"source-table": items["id"], "aggregation": [["count"]], "breakout": [["field", fields["mood"], None]]}}, "count by mood")
    query(
        db,
        {
            "type": "query",
            "query": {
                "source-table": items["id"],
                "filter": [">", ["field", fields["price"], None], 10],
                "aggregation": [["sum", ["field", fields["price"], None]]],
                "breakout": [["field", fields["created_at"], {"temporal-unit": "day"}]],
                "order-by": [["asc", ["field", fields["created_at"], {"temporal-unit": "day"}]]],
            },
        },
        "sum by day",
    )

    # 7. A native query that fails.
    # Metabase answers with the status 400 and the error of the server as JSON text.
    body = json.loads(want(call("POST", "/api/dataset", {"database": db, "type": "native", "native": {"query": "SELECT 1 / 0"}}), "failing query", status=(400,)))
    if body.get("state") != "22012":
        sys.exit(f"want SQLSTATE 22012, got {body.get('state')} {body.get('error')}")
finally:
    server.terminate()
    server.wait(timeout=120)
    log.close()
