# IdleCloud

**A Distributed Cloud Platform Built on Idle Computers**

IdleCloud is a distributed cloud infrastructure that allows anyone to contribute unused computing resources (CPU, RAM, GPU, storage) and enables developers to deploy applications on this decentralized compute network.

The system connects idle machines across different networks and orchestrates them into a unified compute cluster. Developers can deploy applications, background jobs, or services, while the platform automatically schedules workloads on available nodes.

IdleCloud is designed for scalability, security, and performance, with the core infrastructure written in Rust.

---

# Vision

Traditional cloud providers such as Amazon Web Services and Google Cloud rely on massive centralized data centers. However, millions of machines worldwide remain idle for large portions of the day.

IdleCloud aims to convert this unused capacity into a distributed compute cloud where:

* Individuals contribute spare resources.
* Developers deploy applications without managing infrastructure.
* The system automatically schedules workloads across available machines.

The long-term goal is to create a global decentralized compute network capable of running production workloads.

---

# Core Concepts

The platform consists of three primary participants.

## 1. Compute Providers

Users who install the IdleCloud node client and contribute computing resources.

They choose how much CPU, memory, GPU, or storage they want to allocate.

Example:

```
idlecloud join --cpu 4 --ram 8GB
```

Once joined, the machine becomes a worker node capable of executing workloads.

---

## 2. Developers

Developers deploy applications to the platform.

They do not manage servers. Instead, they submit workloads that are scheduled across the distributed network.

Example deployment:

```
idlecloud deploy my-app
```

Applications are packaged as containers and executed on worker nodes.

---

## 3. Platform Scheduler

The central control plane responsible for orchestrating workloads across nodes.

Responsibilities include:

* Node registration
* Resource tracking
* Job scheduling
* Health monitoring
* Network routing
* Security enforcement

---

# High-Level Architecture

```
Developer
   │
   ▼
Deployment CLI
   │
   ▼
Control Plane (Scheduler + API)
   │
   ├─────────────┬─────────────┐
   │             │             │
Node Agent   Node Agent   Node Agent
(Home PC)   (College PC)   (Laptop)
```

Each node runs a lightweight worker agent responsible for executing workloads and reporting resource availability.

---

# System Components

## 1. CLI Tool

The CLI is the primary interface for both developers and compute providers.

### Features

* Node registration
* Resource allocation
* Deployment management
* Cluster monitoring
* Job submission

Example commands:

```
idlecloud join
idlecloud deploy
idlecloud status
idlecloud logs
idlecloud dashboard
idlecloud leave
```

The CLI is distributed as a Rust binary via Cargo.

Installation:

```
cargo install idlecloud
```

---

## 2. Node Agent

The node agent runs on every participating machine.

Responsibilities include:

* Resource monitoring
* Job execution
* Container management
* Health reporting
* Secure sandboxing

Each node periodically sends a heartbeat to the control plane.

Example heartbeat payload:

```
{
  "node_id": "node-129",
  "cpu_available": 6,
  "memory_available": "12GB",
  "gpu": false,
  "uptime": 4821
}
```

---

## 3. Control Plane

The control plane orchestrates the entire system.

Core modules include:

### Node Manager

Tracks all connected nodes and their available resources.

### Scheduler

Assigns workloads to nodes based on:

* CPU availability
* Memory
* latency
* node reliability
* geographic location

### Deployment Manager

Handles application deployments and container lifecycle management.

### Health Monitor

Ensures node reliability using periodic heartbeats.

If a node goes offline, workloads are rescheduled automatically.

---

# Networking Model

IdleCloud nodes may exist behind NAT or firewalls.

To solve this, the system uses an outbound connection model.

Worker nodes initiate secure connections to the control plane.

```
Node → Control Plane
```

The control plane maintains persistent communication channels for job assignment.

Public traffic is routed through gateway servers that proxy requests to nodes running workloads.

---

# Container Runtime

Applications are executed in isolated environments to protect compute providers.

Supported runtimes include:

* Docker containers
* MicroVMs using Firecracker

Each job runs in a sandbox with strict resource limits.

Example container constraints:

```
CPU: 2 cores
Memory: 2GB
Disk: 10GB
Network: restricted
```

---

# Public Routing Layer

Applications deployed on IdleCloud are exposed through a global gateway layer.

```
Internet
   │
   ▼
Gateway / Reverse Proxy
   │
   ▼
Worker Node Running Container
```

The gateway routes traffic based on domain mapping.

Example:

```
myapp.idlecloud.dev → node-23
api.idlecloud.dev → node-51
```

This layer is typically implemented using reverse proxy technologies.

---

# Scalability Design

The platform is designed to support thousands of nodes.

Key scalability strategies include:

## Stateless Control Plane

Most control plane services remain stateless to allow horizontal scaling.

State is stored in distributed databases.

## Message Queues

Job distribution uses queue-based systems to decouple scheduling from execution.

## Sharded Node Registry

Nodes are partitioned across shards to reduce scheduling bottlenecks.

## Regional Gateways

Traffic routing can be distributed across regional edge gateways.

---

# Reliability

IdleCloud must tolerate node failures because worker machines may disconnect frequently.

Reliability strategies include:

### Heartbeat Monitoring

Nodes send periodic health checks.

### Job Replication

Critical workloads can run on multiple nodes.

### Automatic Rescheduling

If a node disappears, jobs are reassigned to another machine.

---

# Security Model

Running arbitrary code on volunteer machines introduces security risks.

Key security measures include:

### Container Isolation

All workloads run inside sandboxed environments.

### Resource Limits

Strict CPU and memory limits prevent abuse.

### Signed Images

Only verified container images are allowed.

### Network Isolation

Containers cannot access the host filesystem or network interfaces.

---

# Rust Tech Stack

IdleCloud is built primarily using Rust for performance and safety.

Core crates used across the project include:

### CLI

* clap
* tokio
* reqwest
* serde
* anyhow

### Node Agent

* sysinfo
* tokio
* tonic (for gRPC)
* tracing

### Control Plane

* axum or actix-web
* sqlx
* redis
* tokio

---

# Example Workflow

### Node Provider

```
cargo install idlecloud
idlecloud join
```

User selects resources to contribute.

The machine becomes part of the network.

---

### Developer Deployment

```
idlecloud deploy ./app
```

Deployment steps:

1. Build container
2. Upload image
3. Scheduler selects node
4. Container starts
5. Public URL generated

---

# Observability

Monitoring is critical for distributed systems.

The platform provides:

* Node metrics
* Resource usage graphs
* Job logs
* Failure alerts

Metrics are collected using telemetry pipelines.

---

# Dashboard

IdleCloud includes a desktop dashboard similar to Docker Desktop.

The UI displays:

* cluster status
* active nodes
* CPU usage
* running jobs
* deployment logs

---

# Future Roadmap

Potential future improvements include:

* decentralized scheduler
* P2P node discovery
* GPU workload support
* edge compute deployments
* marketplace for compute providers

---

# Summary

IdleCloud transforms idle computers into a unified distributed cloud infrastructure.

It enables:

* global compute sharing
* application hosting
* distributed job execution
* decentralized infrastructure

Built with Rust for performance, reliability, and safety, IdleCloud aims to push the boundaries of decentralized cloud computing.
