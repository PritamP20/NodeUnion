# NodeUnion Command Reference

This document captures the commands used to build, run, test, and troubleshoot the current agent + orchestrator + Postgres integration.

## 1) Workspace checks

```bash
pwd
ls
```

## 2) Compile checks

```bash
cargo check -p db
cargo check -p orchestrator
cargo check -p db && cargo check -p orchestrator
```

## 3) Environment setup for Neon/Postgres

Use .env in repo root.

```bash
# zsh safe loading of .env variables
set -a
source .env
set +a
```

If your URL contains query params with &, keep it quoted in .env:

```env
DATABASE_URL="postgresql://.../neondb?sslmode=require&channel_binding=require"
```

## 4) Run services

### Orchestrator

```bash
cargo run -p orchestrator >/tmp/orchestrator.log 2>&1 & echo $!
```

### Agent

```bash
target/debug/agent >/tmp/agent.log 2>&1 & echo $!
```

## 5) Restart orchestrator cleanly

```bash
pkill -f "target/debug/orchestrator|cargo run -p orchestrator" || true
sleep 1
cargo run -p orchestrator >/tmp/orchestrator_neon.log 2>&1 & echo $!
```

## 6) Runtime log checks

```bash
tail -n 40 /tmp/orchestrator.log
tail -n 40 /tmp/orchestrator_neon.log
tail -n 40 /tmp/agent.log
```

## 7) Health checks

```bash
curl -s -o /tmp/health.out -w "%{http_code}" http://127.0.0.1:8080/health && echo && cat /tmp/health.out
```

## 8) Node API tests

### Register node

```bash
curl -s -X POST http://127.0.0.1:8080/nodes/register \
  -H 'content-type: application/json' \
  -d '{"node_id":"db-test-node","agent_url":"http://127.0.0.1:8090","region":"us-east-1","labels":{"tier":"dev"}}'
```

### List nodes

```bash
curl -s http://127.0.0.1:8080/nodes
```

## 9) Job API tests

### Submit job

```bash
curl -s -X POST http://127.0.0.1:8080/jobs/submit \
  -H 'content-type: application/json' \
  -d '{"image":"alpine:3.20","command":["echo","hi"],"cpu_limit":0.25,"ram_limit_mb":128}'
```

### List jobs

```bash
curl -s http://127.0.0.1:8080/jobs
```

## 10) Combined quick smoke test

```bash
curl -s -X POST http://127.0.0.1:8080/nodes/register -H 'content-type: application/json' -d '{"node_id":"db-test-node","agent_url":"http://127.0.0.1:8090","region":"us-east-1","labels":{"tier":"dev"}}' && echo && curl -s http://127.0.0.1:8080/nodes

curl -s -X POST http://127.0.0.1:8080/jobs/submit -H 'content-type: application/json' -d '{"image":"alpine:3.20","command":["echo","hi"],"cpu_limit":0.25,"ram_limit_mb":128}' && echo && curl -s http://127.0.0.1:8080/jobs
```

## 11) Useful process commands

```bash
ps aux | grep orchestrator
ps aux | grep agent
pkill -f orchestrator
pkill -f agent
```

## 12) Git/inspection commands used

```bash
git remote -v
```

## 13) Recommended daily workflow

```bash
# 1) compile
cargo check -p db && cargo check -p orchestrator

# 2) start orchestrator
set -a; source .env; set +a
cargo run -p orchestrator >/tmp/orchestrator.log 2>&1 & echo $!

# 3) start agent
target/debug/agent >/tmp/agent.log 2>&1 & echo $!

# 4) quick test
curl -s http://127.0.0.1:8080/health
curl -s http://127.0.0.1:8080/nodes
curl -s http://127.0.0.1:8080/jobs
```

## 14) SQL migrations (sqlx-cli)

Install sqlx-cli once:

```bash
cargo install sqlx-cli --no-default-features --features postgres,rustls
```

Load env before running migration commands:

```bash
set -a
source .env
set +a
```

Create a new migration file:

```bash
sqlx migrate add init_orchestrator_tables
sqlx migrate add add_retry_metadata
```

Run all pending migrations:

```bash
sqlx migrate run
```

Check migration status:

```bash
sqlx migrate info
```

Revert last migration:

```bash
sqlx migrate revert
```

Force migration version (manual recovery):

```bash
sqlx migrate force <version>
```

## 15) SQLx query metadata generation ("client generation")

For SQLx, there is no Prisma-style generated client. The equivalent is preparing SQL query metadata.

Generate/update SQLx metadata for workspace:

```bash
set -a; source .env; set +a
cargo sqlx prepare --workspace
```

Validate metadata in CI/local checks:

```bash
set -a; source .env; set +a
cargo sqlx prepare --workspace --check
```

## 16) Optional DB reset (local/dev only)

Drop and recreate DB (dangerous; do not run on production):

```bash
set -a; source .env; set +a
sqlx database drop -y
sqlx database create
sqlx migrate run
```

## 17) Recommended migration folder location

Current runtime schema is initialized in code via `init_schema` in the db crate. If you move fully to migration-driven schema, keep SQL files in:

```text
crates/orchestrator/migrations/
```

Then run:

```bash
set -a; source .env; set +a
cd crates/orchestrator
sqlx migrate run
```
