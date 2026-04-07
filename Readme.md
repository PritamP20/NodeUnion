# Idle Resource Utilization Platform

> A distributed local cloud built on Rust — harvesting idle institutional machines (universities, hospitals, government offices) into a shared compute pool, accessible over the internet, with Solana as the decentralized billing and trust layer.

---

## What This Is

Most institutional computers — university lab PCs, hospital workstations, government office machines — sit completely idle for 15+ hours every day. This platform turns that wasted capacity into a real cloud.

A user submits a Docker container. The platform finds an idle machine, runs it there, gives the user a public URL, and keeps it alive by silently migrating the container to another idle machine whenever the current host is about to be reclaimed by its owner. The user never notices the migration. Their app stays online.

No AWS. No GCP. Just idle machines doing useful work.

---

## How It Works — End to End

### 1. A machine goes idle
A Rust daemon runs silently on every institutional machine. It polls CPU and RAM every 30 seconds. When usage drops below 15% for 5 continuous minutes, the machine is considered idle. The daemon registers itself with the orchestrator, reporting available CPU, RAM, disk, and how long the idle window lasts.

### 2. A user deploys a project
The user goes to the dashboard (or uses the CLI), provides a Docker image and resource requirements (e.g. 1 vCPU, 2 GB RAM), and clicks deploy. The orchestrator receives the job, scores all available idle nodes using a bin-packing algorithm, picks the best fit, and dispatches the container to that node's daemon.

### 3. The app goes live
Within ~60 seconds the container is running on an idle university PC. The public reverse proxy (the only always-on component) receives a new routing rule: `myproject.platform.io → 10.0.1.5:32451` over WireGuard. The user gets their public URL. Their app is live on the internet.

### 4. The owner comes back
At 7:45 AM — 15 minutes before the idle window ends — the daemon signals the orchestrator: "this node is draining." The orchestrator finds the next best idle node. CRIU freezes the running container, copies its exact memory state to the new node, restores it there, and updates the proxy routing. The user's URL never changes. The whole migration takes under 30 seconds.

### 5. No node is available
If every node is busy or all idle windows have ended, the container is checkpointed and saved to MinIO. The user's app goes temporarily offline with a notification. The moment a new idle machine registers (e.g. 6 PM when offices close), the container is restored automatically and the app comes back online.

### 6. Billing happens on Solana
When a job starts, the user's tokens are locked in a Solana escrow account. Every 15 minutes the orchestrator posts a transaction debiting tokens proportional to actual CPU and RAM used. When a job ends, the escrow settles. Provider nodes automatically earn tokens for contributing compute. Every billing event is on-chain, immutable, and publicly auditable — no central billing authority.

---

## Architecture

```
[ Browser / CLI ]
       |
       v
[ Public Reverse Proxy ]   ← only always-on component (cheap VPS)
       |  (WireGuard)
       v
[ Orchestrator ]           ← scheduler, metering, migration engine
       |
  _____|_____
 |     |     |
 v     v     v
[Node A] [Node B] [Node C]   ← idle institutional machines
 daemon   daemon   daemon
```

### Components

| Component | What it does |
|---|---|
| **Rust daemon** | Runs on every provider machine. Detects idle, runs containers, sends heartbeats, pushes results |
| **Orchestrator** | Schedules jobs, tracks chunks, triggers migration, runs metering loop |
| **Public proxy** | HAProxy on a VPS. Single stable entry point. Routes traffic to whichever node is currently running a job |
| **WireGuard overlay** | Encrypted private network connecting all nodes and the orchestrator |
| **PostgreSQL** | Persistent store for jobs, nodes, chunks, billing records |
| **Redis** | Fast token balance cache, pub/sub for node events |
| **MinIO** | S3-compatible object store for CRIU checkpoint bundles |
| **Solana programs** | Three Anchor programs: resource registry, billing meter, reputation |
| **Dashboard** | React frontend. Deploy jobs, monitor status, view usage |

---

## Build Order

Build in this exact sequence. Each phase must work end-to-end before the next starts.

### Phase 1 — Rust Daemon

The daemon is a single Rust binary that runs as a `systemd` service on institutional machines.

**What to build:**

1. **Idle detector** — poll CPU/RAM every 30s using `sysinfo`. Sliding window average over 10 samples. Idle threshold: CPU < 15% sustained for 5 minutes. Preemption threshold: CPU > 60% for 2 consecutive polls.

2. **Container manager** — use `bollard` (async Docker API). Launch a container with cgroups CPU and RAM caps. Monitor it. Stop it cleanly on preemption or job completion.

3. **Heartbeat sender** — every 60 seconds, POST to orchestrator with `{ cpu_available, ram_mb, disk_gb, idle_until }`. Three missed heartbeats = orchestrator marks node dead.

4. **Job receiver** — tiny HTTP server (two endpoints):
   - `POST /run` — receives job spec (image, cpu_limit, ram_limit, chunk_id, input_path), launches container
   - `POST /stop` — gracefully stops container, saves partial output to MinIO

5. **Result pusher** — when container exits successfully, upload output to MinIO at `results/{job_id}/{chunk_id}/`. Then POST to orchestrator: `{ chunk_id, status: "done" }`.

6. **Preemption handler** — background task watching CPU usage. On spike: checkpoint with CRIU, upload checkpoint to MinIO, notify orchestrator `{ node_id, reason: "preempted" }`.

**Key crates:** `tokio`, `sysinfo`, `bollard`, `axum`, `reqwest`, `aws-sdk-s3` (for MinIO), `tracing`, `serde`, `anyhow`

**Deliverable:** A binary you can drop on any Ubuntu 22.04+ machine, run `systemctl start idle-agent`, and it registers itself automatically.

---

### Phase 2 — Orchestrator

The orchestrator is a long-running Rust service. It is the brain of the cluster.

**What to build, in order:**

1. **Node registry** — accept daemon registrations, store in PostgreSQL, update resource availability on heartbeat, mark nodes `Offline` after 3 missed heartbeats.

2. **Job intake** — accept `POST /jobs` with (image, vcpu, ram_gb, input_data). Validate, store in DB with status `Pending`, return job UUID.

3. **Chunk splitter** — when a job arrives, split the input into N chunks (one per available node). For files: byte ranges. For datasets: row ranges. Store each chunk in the `chunks` table with status `Pending`.

4. **Scheduler** — for each pending chunk, score every available node:
   ```
   score = (cpu_fit × 30) + (ram_fit × 40) + (disk_fit × 20) + (hours_remaining × stability_bonus) − (reputation_penalty)
   ```
   Pick the highest-scoring node. Write a soft reservation (30s TTL) to prevent double-booking. Dispatch via `POST /run` on the daemon. On confirmation: convert reservation to allocation.

5. **Chunk tracker** — background task polling chunk statuses. If a chunk is `Running` but its node goes `Offline`: re-dispatch to another node. If a chunk completes: mark `Done`, check if all chunks for the job are done.

6. **Result merger** — when all chunks are `Done`: pull from MinIO, merge, mark job `Complete`, notify user.

7. **Metering loop** — every 60s, query each daemon for actual resource usage of running containers. Accumulate in `job_usage` table. Every 15 minutes: submit Solana billing transaction.

8. **Drain manager** — background task. Nodes with `idle_window_end < now + 30 minutes` get status `Draining`. For each job on a draining node: find next best node, trigger CRIU migration sequence.

**Key crates:** `axum`, `sqlx`, `tokio`, `dashmap`, `reqwest`, `solana-client`, `tracing`

---

### Phase 3 — API Gateway + Dashboard

Thin `axum` layer over the orchestrator. No business logic here.

**Endpoints:**
- `POST /jobs` — submit a job
- `GET /jobs/{id}` — status, endpoint URL, resource usage
- `DELETE /jobs/{id}` — stop job, trigger final billing
- `GET /balance` — user's token balance from Solana
- `GET /nodes` — admin: cluster capacity overview
- `WS /jobs/{id}/logs` — stream container logs in real time

**Dashboard (React):**
- Job submission form (image URL, CPU, RAM sliders)
- Live cluster map (which nodes are online, which are running what)
- Job status page with public URL and usage graph
- Token balance and top-up

---

### Phase 4 — Networking + Security

**WireGuard overlay:**
- Orchestrator acts as WireGuard hub
- Each daemon gets a WireGuard config on registration with a `/24` private IP
- All orchestrator↔daemon traffic goes over WireGuard, never public internet

**mTLS:**
- Generate certificates with `rcgen`
- All gRPC calls between orchestrator and daemons use mTLS
- Rotate certificates every 30 days

**LUKS encryption:**
- On first daemon startup: create an encrypted partition with `cryptsetup`
- Keys stored in HashiCorp Vault
- Daemon fetches key on startup, mounts partition, zeroes key from memory with `zeroize`
- User containers run inside this encrypted partition — zero access to host filesystem

**Keycloak SSO:**
- Connect to institutional Active Directory / LDAP
- OAuth2/SAML login for the dashboard
- Role-based access: Admin, Operator, User

---

### Phase 5 — Solana Layer

Three Anchor programs deployed on Solana. Add this layer only after the core platform works end-to-end on at least 2 real nodes.

**Program 1: Resource Registry**
- `register_node(node_id, institution, vcpu, ram_gb, disk_gb)` — called by daemon on first registration
- `update_stats(node_id, jobs_completed, uptime_score)` — called by orchestrator periodically
- On-chain source of truth for which nodes exist and their reputation

**Program 2: Billing Meter**
- `open_escrow(job_id, user_pubkey, token_amount)` — called when user submits job. Locks SPL tokens.
- `record_usage(job_id, vcpu_seconds, ram_mb_seconds)` — called by orchestrator every 15 minutes. Debits from escrow, credits provider wallet.
- `close_escrow(job_id)` — called on job completion. Settles remaining balance.
- `suspend_job(job_id)` — pauses billing when job is queued with no available nodes.

**Program 3: Reputation**
- `slash_node(node_id, reason)` — called when a node drops a job unexpectedly. Reduces uptime score.
- `reward_node(node_id)` — called on successful job migration or completion. Increases uptime score.
- Reputation score used by scheduler to penalize unreliable nodes.

**Off-chain Solana client (Rust):**
- `solana-client` + `solana-sdk` for submitting transactions
- Orchestrator's signing keypair loaded from env var — never hardcoded
- Batch multiple `record_usage` calls into one versioned transaction using Address Lookup Tables
- Retry with priority fees on `TransactionError`

---

## Distributed Workload Splitting

For computationally heavy jobs, the orchestrator splits input across multiple nodes.

**Embarrassingly parallel jobs** (80% of use cases — do this first):
```
Job: process 10GB dataset
→ Node A: rows 0–2.5M
→ Node B: rows 2.5M–5M
→ Node C: rows 5M–7.5M
→ Node D: rows 7.5M–10M
→ Orchestrator: merge results from MinIO
```
Each node is independent. No inter-node communication. Results merged after all chunks complete.

**Failure handling:**
- Every chunk has a status: `Pending | Running | Done | Failed`
- If a chunk's node goes offline mid-execution: re-dispatch to next available node
- If no node available: park chunk as `Suspended`, retry when capacity returns
- Partial outputs saved to MinIO before preemption so work is not lost

---

## Technology Stack

| Layer | Technology | Why |
|---|---|---|
| Daemon + Orchestrator | Rust + tokio | Performance, safety, single binary deployment |
| HTTP API | axum + tower | Async, composable middleware |
| Container management | bollard | Async Docker API, no CLI shelling |
| System metrics | sysinfo | Cross-platform CPU/RAM polling |
| Database | PostgreSQL + sqlx | Compile-time checked queries |
| Cache + pub/sub | Redis + redis-rs | Fast node event propagation |
| Object storage | MinIO (S3-compatible) | CRIU checkpoint storage, self-hosted |
| Overlay network | WireGuard | Encrypted, fast, simple |
| Reverse proxy | HAProxy | Dynamic backend management via stats socket |
| Encryption | LUKS + HashiCorp Vault | Host partition encryption, KMS |
| Auth | Keycloak | OAuth2/SAML, institutional SSO |
| Blockchain | Solana + Anchor | Decentralized billing and trust |
| Token standard | SPL Token | Native Solana fungible token |
| Frontend | React + Tailwind | Dashboard and job management UI |
| Monitoring | Prometheus + Grafana | Cluster health, billing metrics |

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Node goes offline mid-job | High | CRIU checkpoint before preemption, re-dispatch chunk |
| No nodes available | Medium | Checkpoint to MinIO, queue job, restore when capacity returns |
| CRIU migration fails | High | Fallback: suspend job, notify user, retry on next node |
| Solana RPC down | Medium | Retry with backoff, buffer billing records in Postgres |
| Host data breach | Critical | LUKS encryption + container namespace isolation |
| Double-booking a node | Medium | Soft reservation with 30s TTL, atomic DB update |
| Daemon crashes | Low | systemd Restart=always, state persisted in DB not in-process |
| Token balance hits zero | Medium | Grace period warning, auto-suspend with checkpoint |

---

## Project Structure

```
idle-resource-platform/
├── Cargo.toml                    # workspace root
├── crates/
│   ├── common/                   # shared types, errors, proto definitions
│   ├── agent/                    # the Rust daemon
│   │   ├── idle_detector/
│   │   ├── container_manager/
│   │   ├── heartbeat/
│   │   ├── job_receiver/
│   │   └── result_pusher/
│   ├── orchestrator/             # scheduler, metering, migration
│   │   ├── node_registry/
│   │   ├── scheduler/
│   │   ├── chunk_tracker/
│   │   ├── metering/
│   │   ├── drain_manager/
│   │   └── api/
│   └── solana-client/            # off-chain Solana interaction
├── programs/                     # Anchor on-chain programs
│   ├── resource-registry/
│   ├── billing-meter/
│   └── reputation/
├── dashboard/                    # React frontend
├── deploy/
│   ├── agent.service             # systemd unit file
│   ├── docker-compose.yml        # orchestrator + infra stack
│   └── wireguard/                # WireGuard config templates
└── README.md
```

---

## Getting Started (Development)

```bash
# 1. Clone and build
git clone https://github.com/yourorg/idle-resource-platform
cd idle-resource-platform
cargo build --workspace

# 2. Start infrastructure (Postgres, Redis, MinIO)
docker compose -f deploy/docker-compose.yml up -d

# 3. Run the orchestrator
cargo run -p orchestrator

# 4. Run a daemon (on the same machine for local testing)
cargo run -p agent -- --orchestrator http://localhost:8080

# 5. Submit a test job
curl -X POST http://localhost:8080/jobs \
  -H "Content-Type: application/json" \
  -d '{"image": "alpine", "vcpu": 1, "ram_mb": 512}'
```

---

## What the User Sees

1. Sign in with institutional email
2. Upload Docker image or paste image URL
3. Set CPU and RAM requirements
4. Click Deploy
5. Get a public URL within 60 seconds
6. App stays online — migrates silently between idle machines
7. Pay only for actual compute time, billed in IDLE tokens on Solana
8. When no machines are available, app queues and auto-resumes

---

*Built with Rust. Runs on machines that would otherwise be doing nothing.*