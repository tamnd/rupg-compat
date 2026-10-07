"""The trace scenario of spec/21 section 21.4.2, with Grafana and its PostgreSQL data source.

A BI tool does not send the same statements as a driver. Grafana runs the SQL in its backend, but the web pages of Grafana make the SQL of the query editor. So the scenario sends the requests of the web pages: the health check of the data source, the version and TimescaleDB checks of the configuration page, the table and column lists of the query editor (postgresMetaQuery.ts of the plugin), a time series query with macros, a table query and a query that fails.
"""

import base64
import json
import os
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request

home = sys.argv[1]
work = os.path.dirname(home)

# A free port on the loopback for the HTTP server of Grafana.
with socket.socket() as s:
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]

for d in ("data", "logs", "plugins", "provisioning/datasources"):
    os.makedirs(os.path.join(work, d), exist_ok=True)

with open(os.path.join(work, "provisioning/datasources/compat.yaml"), "w") as f:
    json.dump(
        {
            "apiVersion": 1,
            "datasources": [
                {
                    "name": "compat",
                    "uid": "compat",
                    "type": "grafana-postgresql-datasource",
                    "url": f"{os.environ['PGHOST']}:{os.environ['PGPORT']}",
                    "user": os.environ["PGUSER"],
                    "secureJsonData": {"password": os.environ["PGPASSWORD"]},
                    "jsonData": {
                        "database": os.environ["PGDATABASE"],
                        "sslmode": "disable",
                        "maxOpenConns": 1,
                        "maxIdleConns": 1,
                        "connMaxLifetime": 14400,
                    },
                }
            ],
        },
        f,
    )

env = dict(
    os.environ,
    GF_SERVER_HTTP_ADDR="127.0.0.1",
    GF_SERVER_HTTP_PORT=str(port),
    GF_PATHS_DATA=os.path.join(work, "data"),
    GF_PATHS_LOGS=os.path.join(work, "logs"),
    GF_PATHS_PLUGINS=os.path.join(work, "plugins"),
    GF_PATHS_PROVISIONING=os.path.join(work, "provisioning"),
    GF_SECURITY_ADMIN_PASSWORD="compat",
    GF_ANALYTICS_REPORTING_ENABLED="false",
    GF_ANALYTICS_CHECK_FOR_UPDATES="false",
    GF_ANALYTICS_CHECK_FOR_PLUGIN_UPDATES="false",
    GF_NEWS_NEWS_FEED_ENABLED="false",
    GF_LOG_MODE="file",
)
server = subprocess.Popen(
    [os.path.join(home, "bin/grafana"), "server", "--homepath", home],
    env=env,
    stdout=subprocess.DEVNULL,
    stderr=subprocess.DEVNULL,
)
auth = "Basic " + base64.b64encode(b"admin:compat").decode()


def call(path, body=None):
    req = urllib.request.Request(
        f"http://127.0.0.1:{port}{path}",
        data=None if body is None else json.dumps(body).encode(),
        headers={"Authorization": auth, "Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=60) as r:
            return r.status, json.load(r)
    except urllib.error.HTTPError as e:
        return e.code, json.load(e)


# The time range of the queries, in milliseconds. The rows of schema.sql are in it.
FROM, TO = "1767322800000", "1767323400000"


def query(sql, fmt="table", ref="A"):
    """Runs one query, as the query editor does. One query for each request keeps the order of the statements."""
    target = {"refId": ref, "datasource": {"type": "grafana-postgresql-datasource", "uid": "compat"}, "rawSql": sql, "format": fmt, "rawQuery": True}
    return call("/api/ds/query", {"from": FROM, "to": TO, "queries": [target]})


def ok(result, what):
    status, body = result
    frames = body.get("results", {})
    if status != 200 or any("error" in r for r in frames.values()):
        sys.exit(f"{what}: {status} {body}")
    return frames


try:
    for _ in range(120):
        try:
            if call("/api/health")[0] == 200:
                break
        except OSError:
            pass
        time.sleep(1)
    else:
        sys.exit("Grafana did not start")

    # 1. The health check of the data source, from the "Save & test" button.
    status, body = call("/api/datasources/uid/compat/health")
    if status != 200 or body.get("status") != "OK":
        sys.exit(f"health: {status} {body}")

    # 2. The version and TimescaleDB checks of the configuration page.
    ok(query("SELECT current_setting('server_version_num')::int/100 as version"), "version")
    ok(query("SELECT extversion FROM pg_extension WHERE extname = 'timescaledb'"), "timescaledb")

    # 3. The table and column lists of the query editor.
    schema = """
          quote_ident(table_schema) IN (
          SELECT
            CASE WHEN trim(s[i]) = '"$user"' THEN user ELSE trim(s[i]) END
          FROM
            generate_series(
              array_lower(string_to_array(current_setting('search_path'),','),1),
              array_upper(string_to_array(current_setting('search_path'),','),1)
            ) as i,
            string_to_array(current_setting('search_path'),',') s
          )"""
    ok(
        query(
            f"""SELECT
    CASE WHEN {schema}
      THEN quote_ident(table_name)
      ELSE quote_ident(table_schema) || '.' || quote_ident(table_name)
    END AS "table"
    FROM information_schema.tables
    WHERE quote_ident(table_schema) NOT IN ('information_schema',
                             'pg_catalog',
                             '_timescaledb_cache',
                             '_timescaledb_catalog',
                             '_timescaledb_internal',
                             '_timescaledb_config',
                             'timescaledb_information',
                             'timescaledb_experimental')
    ORDER BY CASE WHEN {schema} THEN 0 ELSE 1 END, 1""",
            ref="tables",
        ),
        "tables",
    )
    table = "'compat_metrics'"
    ok(
        query(
            f"""SELECT quote_ident(column_name) AS "column", data_type AS "type"
    FROM information_schema.columns
    WHERE
      CASE WHEN array_length(parse_ident({table}),1) = 2
        THEN quote_ident(table_schema) = (parse_ident({table}))[1]
          AND quote_ident(table_name) = (parse_ident({table}))[2]
        ELSE quote_ident(table_name) = {table}
          AND {schema}
      END""",
            ref="columns",
        ),
        "columns",
    )

    # 4. A time series query with macros.
    ok(
        query(
            "SELECT $__timeGroupAlias(\"time\", '1m'), host AS metric, avg(value) AS value FROM compat_metrics WHERE $__timeFilter(\"time\") GROUP BY 1, 2 ORDER BY 1, 2",
            fmt="time_series",
        ),
        "time series",
    )

    # 5. A table query.
    ok(query("SELECT \"time\", host, value FROM compat_metrics WHERE $__timeFilter(\"time\") ORDER BY \"time\", host"), "table")

    # 6. A query that fails.
    status, body = query("SELECT 1 / 0")
    error = json.dumps(body)
    if "22012" not in error and "division by zero" not in error:
        sys.exit(f"want division by zero, got {status} {body}")
finally:
    server.terminate()
    server.wait(timeout=60)
