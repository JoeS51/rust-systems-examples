#!/bin/bash
# Runs once, on first init of the primary's data volume.
set -e

psql -v ON_ERROR_STOP=1 --username postgres <<'SQL'
CREATE ROLE replicator WITH REPLICATION LOGIN PASSWORD 'replicator';
SQL

echo "host replication replicator all scram-sha-256" >> "$PGDATA/pg_hba.conf"
