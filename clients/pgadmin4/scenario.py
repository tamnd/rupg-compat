"""The trace scenario of spec/21 section 21.4.2, with pgAdmin 4.

pgAdmin makes its SQL in its backend from the requests of its web pages. So the scenario starts pgAdmin in desktop mode and sends those requests: the dialog that adds a server and connects to it, the browser tree down to the columns, the properties and the SQL of a table, and the query tool with a query that works and a query that fails. Then it stops pgAdmin.
"""

import http.cookiejar
import json
import os
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request

env, home = sys.argv[1], sys.argv[2]
package = os.path.join(env, "lib", "python%d.%d" % sys.version_info[:2], "site-packages", "pgadmin4")

# A free port on the loopback for the HTTP server of pgAdmin.
with socket.socket() as s:
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]

# pgAdmin reads this file in place of config_distro.py. Desktop mode has one user and no login.
config = os.path.join(home, "config_compat.py")
with open(config, "w") as f:
    f.write(
        f"""SERVER_MODE = False
DATA_DIR = {home!r}
DEFAULT_SERVER = "127.0.0.1"
DEFAULT_SERVER_PORT = {port}
MASTER_PASSWORD_REQUIRED = False
USE_OS_SECRET_STORAGE = False
UPGRADE_CHECK_ENABLED = False
CONSOLE_LOG_LEVEL = 40
"""
    )
log = open(os.path.join(home, "pgadmin.log"), "w")
server = subprocess.Popen(
    [os.path.join(env, "bin", "python"), os.path.join(package, "pgAdmin4.py")],
    cwd=home,
    env=dict(os.environ, HOME=home, CONFIG_DISTRO_FILE_PATH=config),
    stdout=log,
    stderr=subprocess.STDOUT,
)
opener = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
csrf = None


def call(method, path, body=None):
    headers = {"Content-Type": "application/json"}
    if csrf:
        headers["X-pgA-CSRFToken"] = csrf
    req = urllib.request.Request(
        f"http://127.0.0.1:{port}{path}",
        method=method,
        data=None if body is None else json.dumps(body).encode(),
        headers=headers,
    )
    try:
        with opener.open(req, timeout=300) as r:
            return r.status, r.read().decode()
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode()


def want(method, path, body=None):
    code, text = call(method, path, body)
    if code != 200:
        sys.exit(f"{method} {path}: {code} {text[:2000]}")
    return json.loads(text)


def nodes(path):
    return {n["label"]: n["_id"] for n in want("GET", path)["data"]}


def run(trans, sql):
    """Runs one statement in the query tool and polls until it ends. A poll gets the status 500 when the statement fails."""
    want("POST", f"/sqleditor/query_tool/start/{trans}", {"sql": sql})
    while True:
        code, text = call("GET", f"/sqleditor/poll/{trans}")
        body = json.loads(text)
        if code == 500:
            return body
        if code != 200:
            sys.exit(f"poll: {code} {text[:2000]}")
        if body["data"]["status"] not in ("Busy", "NotInitialised"):
            return body["data"]
        time.sleep(0.2)


try:
    for _ in range(300):
        try:
            if call("GET", "/misc/ping")[0] == 200:
                break
        except OSError:
            pass
        if server.poll() is not None:
            sys.exit("pgAdmin stopped")
        time.sleep(1)
    else:
        sys.exit("pgAdmin did not start")

    # The first page logs in the desktop user. The CSRF token is in a script of the browser.
    call("GET", "/")
    code, text = call("GET", "/browser/js/utils.js")
    marker = "pgAdmin['csrf_token'] = '"
    csrf = text[text.index(marker) + len(marker):].split("'", 1)[0]

    # 1. The dialog that adds a server, with "Connect now" and a database restriction.
    group = want("GET", "/browser/server_group/nodes/")["data"][0]["_id"]
    form = {
        "name": "compat",
        "host": os.environ["PGHOST"],
        "port": int(os.environ["PGPORT"]),
        "db": os.environ["PGDATABASE"],
        "username": os.environ["PGUSER"],
        "password": os.environ["PGPASSWORD"],
        "role": None,
        "service": None,
        "connect_now": True,
        "save_password": False,
        "connection_params": [{"name": "sslmode", "value": "disable", "keyword": "sslmode"}],
        # The oracle can have the databases of other suites. The restriction of the dialog keeps them out of the tree, so the trace does not depend on them.
        "db_res": [os.environ["PGDATABASE"]],
        "db_res_type": "databases",
    }
    sid = want("POST", f"/browser/server/obj/{group}/", form)["node"]["_id"]
    want("POST", f"/browser/server/connect/{group}/{sid}", {"password": os.environ["PGPASSWORD"]})

    # 2. The browser tree: the databases, the schemas, the tables and the columns of a table.
    did = nodes(f"/browser/database/nodes/{group}/{sid}/")[os.environ["PGDATABASE"]]
    want("POST", f"/browser/database/connect/{group}/{sid}/{did}")
    scid = nodes(f"/browser/schema/nodes/{group}/{sid}/{did}/")["public"]
    tid = nodes(f"/browser/table/nodes/{group}/{sid}/{did}/{scid}/")["compat_items"]
    nodes(f"/browser/column/nodes/{group}/{sid}/{did}/{scid}/{tid}/")

    # 3. The properties and the SQL tab of the table.
    want("GET", f"/browser/table/obj/{group}/{sid}/{did}/{scid}/{tid}")
    want("GET", f"/browser/table/sql/{group}/{sid}/{did}/{scid}/{tid}")

    # 4. The query tool: a query that works and a query that fails.
    trans = 4242
    want("POST", f"/sqleditor/initialize/sqleditor/{trans}/{group}/{sid}/{did}", {"user": os.environ["PGUSER"], "role": None, "dbname": os.environ["PGDATABASE"]})
    result = run(trans, "SELECT name, price FROM compat_items WHERE price > 3 ORDER BY id")
    if result["status"] != "Success" or result["result"] != [["item 3", "4.50"], ["item 4", "6.00"], ["item 5", "7.50"]]:
        sys.exit(f"query tool: {json.dumps(result)[:2000]}")
    result = run(trans, "SELECT 1 / 0")
    if "SQL state: 22012" not in str(result.get("errormsg")):
        sys.exit(f"want SQLSTATE 22012, got {json.dumps(result)[:2000]}")
    # The scenario does not open the dashboard. Its answers show the sessions, the locks and the counters of the whole server at that time. With the dashboard, 3 of 8 replays had a group of pg_stat_activity that the unstable filter of spec/21 section 21.1 did not remove. The scenario does not close the query tool and does not disconnect the server. pgAdmin does both with a new connection that sends pg_cancel_backend with the process ID of the query tool connection as a literal in the SQL, and a replay cannot give the same process ID. So the scenario stops pgAdmin, as a user who closes the desktop application.
finally:
    server.terminate()
    server.wait(timeout=60)
    log.close()
