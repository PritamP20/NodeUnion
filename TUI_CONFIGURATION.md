# NodeUnion TUI Configuration Reference

Quick reference guide with all configuration values needed to fill the launch TUIs locally.

## 🎯 Quick Reference

### Stellar Account Details
| Item | Value |
|------|-------|
| **Account Name** | `nodeunion-test` |
| **Public Key (Wallet Address)** | `GDWUXVSRVGYT3RDBH26GSZMN6MCKTJGJ2AUYSUAWXK2YRCF53VVPSHOQ` |
| **Network** | `TESTNET_FUTURE` |

### Stellar Contract
| Item | Value |
|------|-------|
| **Contract ID** | `CD7ODCD2U3WAPUOZLL7MJZ4D6IUVBXWPVD5LBA7SNA3S4XJN4KL4RHGK` |
| **Contract ID** | `CC5DFOTE24IDJPFL5IV4647TAAZYCOCJEO4UR76SZPFIBTCTBKPXKV2K` |
| **Network** | Stellar Mainnet |
| **Status** | Deployed ✅ |

### Database
| Item | Value |
|------|-------|
| **Local PostgreSQL** | `postgres://nodeunion:nodeunion@localhost:5432/nodeunion` |
| **Remote (Neon)** | Set your own Neon connection string |

### Services
| Service | Bind Address | Port |
|---------|-------------|------|
| **Orchestrator** | `0.0.0.0:8080` | 8080 |
| **Agent** | `0.0.0.0:8090` | 8090 |
| **Dashboard** | `localhost:3000` | 3000 |

---

## 📋 Orchestrator Launch TUI Fill Guide

Run this to start:
```bash
nodeunion-orchestrator-launch-tui
```

### Prompts & Values to Enter:

#### 1. DATABASE_URL (Required)
**What it is:** PostgreSQL connection string for storing networks, nodes, jobs, and settlements.

**For Local Testing:**
```
postgres://nodeunion:nodeunion@localhost:5432/nodeunion
```

**For Remote (Neon):**
```
postgresql://USER:PASSWORD@HOST:5432/DBNAME?sslmode=require
```

#### 2. STELLAR_NETWORK
**What it is:** Which Stellar network to use.

**Value:**
```
TESTNET_FUTURE
```

#### 3. STELLAR_SOURCE_ACCOUNT (Required)
**What it is:** Your Stellar account public key that will sign transactions and pay for on-chain billing.

**Value:**
```
GDWUXVSRVGYT3RDBH26GSZMN6MCKTJGJ2AUYSUAWXK2YRCF53VVPSHOQ
```

#### 4. STELLAR_CONTRACT_ID (Required)
**What it is:** The deployed Soroban contract ID that handles billing and settlements.

**Value:**
```
CC5DFOTE24IDJPFL5IV4647TAAZYCOCJEO4UR76SZPFIBTCTBKPXKV2K
```

#### 5. STELLAR_RATE_PER_UNIT
**What it is:** Price per compute unit in the billing contract (stroops or smallest unit).

**Default/Value:**
```
100
```

#### 6. ORCHESTRATOR_BIND_ADDR
**What it is:** Address and port the orchestrator HTTP server will bind to.

**Default (Accept):**
```
0.0.0.0:8080
```

#### 7. ORCHESTRATOR_NETWORK_ID (Optional)
**What it is:** If set, the orchestrator operates in single-network mode (only serves this network).

**For Testing:**
```
(leave blank for multi-network mode, or enter a network ID like "network-1")
```

#### 8. ORCHESTRATOR_NETWORK_NAME (If using ORCHESTRATOR_NETWORK_ID)
**What it is:** Human-readable name for the managed network.

**Example:**
```
Test Network
```

#### 9. Open Dashboard Browser?
**What it is:** Whether to automatically open the dashboard web UI.

**Value:**
```
yes (or y)
```

---

## 📋 Agent Launch TUI Fill Guide

Run this to start (in a new terminal):
```bash
nodeunion-agent-launch-tui
```

### Prompts & Values to Enter:

#### 1. ORCHESTRATOR_BASE_URL (Required)
**What it is:** HTTP endpoint of the running orchestrator.

**For Local Testing:**
```
http://localhost:8080
```

**For Remote/Tunnel:**
```
https://your-cloudflare-tunnel-url.trycloudflare.com
```

#### 2. NODE_ID (Required)
**What it is:** Unique identifier for this compute node.

**Example Values:**
```
1
node-1
provider-machine-1
```

#### 3. NETWORK_ID (From Dropdown)
**What it is:** Select which network this agent should operate in.

**Available Networks:** The TUI will fetch networks from the orchestrator. Select one from the list.

**Or Enter Manually:**
```
default
network-1
test-network
```

#### 4. PROVIDER_WALLET (Required)
**What it is:** Your Stellar wallet address for receiving payments when jobs are executed.

**Value (Same as Orchestrator Account):**
```
GDWUXVSRVGYT3RDBH26GSZMN6MCKTJGJ2AUYSUAWXK2YRCF53VVPSHOQ
```

#### 5. AGENT_BIND_ADDR
**What it is:** Address and port the agent HTTP server will bind to.

**Default (Accept):**
```
0.0.0.0:8090
```

#### 6. AGENT_PUBLIC_URL (Optional)
**What it is:** Public URL if agent is behind a firewall/tunnel. Leave blank for auto-detection.

**For Local Network:**
```
(leave blank, auto-detect)
```

**For Remote/Tunnel:**
```
https://your-agent-tunnel-url.trycloudflare.com
```

#### 7. Heartbeat Interval
**What it is:** How often agent sends health status to orchestrator (seconds).

**Default (Accept):**
```
10
```

#### 8. Job Timeout
**What it is:** Max seconds to wait for a container job to complete.

**Default (Accept):**
```
3600
```

#### 9. Docker Socket
**What it is:** Path to Docker daemon socket.

**Default (Accept):**
```
unix:///var/run/docker.sock
```

#### 10. Enable Agent Idle Detection?
**What it is:** Whether to detect and report idle status.

**Default (Accept):**
```
yes (or y)
```

#### 11. Idle Detection Interval
**What it is:** How often to check for idle (seconds).

**Default (Accept):**
```
30
```

#### 12. Idle Threshold
**What it is:** Seconds without activity before marking as idle.

**Default (Accept):**
```
60
```

#### 13. Open Local Agent TUI?
**What it is:** Whether to open the agent monitoring dashboard in a terminal.

**Value:**
```
yes (or y)
```

---

## 🗄️ Database Setup (Local)

If using local PostgreSQL:

```bash
# Start PostgreSQL in Docker
docker run -d \
  --name nodeunion-postgres \
  -e POSTGRES_USER=nodeunion \
  -e POSTGRES_PASSWORD=nodeunion \
  -e POSTGRES_DB=nodeunion \
  -p 5432:5432 \
  postgres:15

# Wait a few seconds for the database to initialize
sleep 5

# Run migrations (orchestrator will auto-initialize schema)
# Just make sure the database exists and is accessible
```

---

## 🚀 Full Launch Sequence

**Terminal 1 - Start Orchestrator:**
```bash
nodeunion-orchestrator-launch-tui
```
Fill with values from **Orchestrator Launch TUI Fill Guide** above.

**Terminal 2 - Start Agent (after orchestrator is ready):**
```bash
nodeunion-agent-launch-tui
```
Fill with values from **Agent Launch TUI Fill Guide** above.

**Terminal 3 - Monitor Orchestrator:**
```bash
nodeunion-orchestrator-local-tui
```

**Terminal 4 - Monitor Agent:**
```bash
nodeunion-agent-local-tui
```

**Terminal 5 - Test Services:**
```bash
# Check orchestrator health
curl http://localhost:8080/health

# Check agent health
curl http://localhost:8090/health
```

---

## 🔧 Manual Environment Variables

If you prefer not to use the TUI, you can set environment variables directly:

### Orchestrator
```bash
export DATABASE_URL="postgres://nodeunion:nodeunion@localhost:5432/nodeunion"
export STELLAR_NETWORK="TESTNET_FUTURE"
export STELLAR_SOURCE_ACCOUNT="GDWUXVSRVGYT3RDBH26GSZMN6MCKTJGJ2AUYSUAWXK2YRCF53VVPSHOQ"
export STELLAR_CONTRACT_ID="CC5DFOTE24IDJPFL5IV4647TAAZYCOCJEO4UR76SZPFIBTCTBKPXKV2K"
export STELLAR_RATE_PER_UNIT="100"
export ORCHESTRATOR_BIND_ADDR="0.0.0.0:8080"

nodeunion-orchestrator
```

### Agent
```bash
export NODE_ID="1"
export NETWORK_ID="default"
export ORCHESTRATOR_BASE_URL="http://localhost:8080"
export PROVIDER_WALLET="GDWUXVSRVGYT3RDBH26GSZMN6MCKTJGJ2AUYSUAWXK2YRCF53VVPSHOQ"
export AGENT_BIND_ADDR="0.0.0.0:8090"

nodeunion-agent
```

---

## ✅ Verification Checklist

- [ ] Stellar account `nodeunion-test` is funded
- [ ] PostgreSQL is running and accessible
- [ ] Port 8080 is available for orchestrator
- [ ] Port 8090 is available for agent
- [ ] Docker daemon is running
- [ ] Binaries are in PATH: `which nodeunion-orchestrator`
- [ ] Orchestrator starts successfully and shows "listening on 0.0.0.0:8080"
- [ ] Agent registers successfully with orchestrator
- [ ] Both `/health` endpoints return 200 OK

---

## 🆘 Troubleshooting

### "Database connection refused"
- Ensure PostgreSQL is running: `docker ps | grep postgres`
- Verify connection string is correct: `psql postgres://nodeunion:nodeunion@localhost:5432/nodeunion`

### "Failed to register node with orchestrator: 500"
- Ensure orchestrator is running and responding to HTTP requests
- Check orchestrator logs for the actual error
- Verify ORCHESTRATOR_BASE_URL is correct

### "Port already in use"
- Agent/orchestrator prompts will suggest alternate free ports
- Or kill the process: `lsof -i :8080` then `kill <PID>`

### "Docker socket not accessible"
- Ensure Docker daemon is running: `docker ps`
- Verify socket permissions: `ls -l /var/run/docker.sock`

### "Stellar account not funded"
- Go to https://friendbot.stellar.org/
- Enter your public key: `GDWUXVSRVGYT3RDBH26GSZMN6MCKTJGJ2AUYSUAWXK2YRCF53VVPSHOQ`
- Click "Fund"

---

## 📚 Additional Resources

- [Stellar CLI Docs](https://developers.stellar.org/tools/cli)
- [Soroban Contract Reference](https://soroban.stellar.org/)
- [PostgreSQL Connection Strings](https://www.postgresql.org/docs/current/libpq-connect.html)
- Local deployment guide: [LOCAL_DEPLOYMENT.md](LOCAL_DEPLOYMENT.md)

