r"""Adds the tenant of the Supavisor scenario with the HTTP API of Supavisor.

The API takes a JWT that is signed with API_JWT_SECRET. The tenant is the oracle behind the proxy, in transaction mode, with a pool of two server connections.

Before it adds a tenant, the API connects to the server with Postgrex and reads version(). Supavisor 2.9.13 finds the version with the pattern "PostgreSQL (\d+\.\d+)", so it rejects a server with the version 19beta4. For that error only, this script writes the tenant to the file of argv[2] with the server_version that follows "PostgreSQL " in version(), and exits with the status 3. Then run.sh adds the tenant with tenant.exs.
"""

import base64
import hashlib
import hmac
import json
import os
import sys
import time
import urllib.error
import urllib.request

port, fallback = sys.argv[1], sys.argv[2]


def b64(data):
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode()


header = b64(json.dumps({"alg": "HS256", "typ": "JWT"}).encode())
claims = b64(json.dumps({"iss": "compat", "role": "anon", "iat": 1767225600, "exp": 4102444800}).encode())
signature = b64(hmac.new(os.environ["API_JWT_SECRET"].encode(), f"{header}.{claims}".encode(), hashlib.sha256).digest())
token = f"{header}.{claims}.{signature}"

tenant = {
    "tenant": {
        "db_host": os.environ["PGHOST"],
        "db_port": int(os.environ["PGPORT"]),
        "db_database": os.environ["PGDATABASE"],
        "ip_version": "v4",
        "enforce_ssl": False,
        "require_user": True,
        "upstream_ssl": False,
        "default_pool_size": 2,
        "users": [
            {
                "db_user": os.environ["PGUSER"],
                "db_password": os.environ["PGPASSWORD"],
                "pool_size": 2,
                "mode_type": "transaction",
                "is_manager": True,
            }
        ],
    }
}

for _ in range(120):
    req = urllib.request.Request(
        f"http://127.0.0.1:{port}/api/tenants/compat",
        method="PUT",
        data=json.dumps(tenant).encode(),
        headers={"Authorization": f"Bearer {token}", "Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            if r.status in (200, 201):
                break
    except urllib.error.HTTPError as e:
        error = e.read().decode()
        prefix = "Can't parse version in PostgreSQL "
        if e.code == 400 and prefix in error:
            version = error.split(prefix, 1)[1].split()[0]
            tenant["tenant"].update(external_id="compat", default_parameter_status={"server_version": version})
            with open(fallback, "w") as f:
                json.dump(tenant["tenant"], f)
            print(f"Supavisor rejects the version {version}, so tenant.exs adds the tenant", file=sys.stderr)
            sys.exit(3)
        sys.exit(f"tenant: {e.code} {error[:2000]}")
    except OSError:
        pass
    time.sleep(1)
else:
    sys.exit("Supavisor did not start")
