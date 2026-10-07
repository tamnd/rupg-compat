-- The table that the Grafana scenario reads.
CREATE TABLE compat_metrics ("time" timestamptz NOT NULL, host text NOT NULL, value float8);
INSERT INTO compat_metrics SELECT '2026-01-02 03:00:00+00'::timestamptz + i * interval '20 seconds', h, i * 1.5 FROM generate_series(0, 29) AS i, unnest(ARRAY['a', 'b']) AS h;
