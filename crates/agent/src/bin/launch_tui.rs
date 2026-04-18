use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};

fn prompt(label: &str, env_key: &str, default: &str) -> String {
    let current = env::var(env_key).unwrap_or_else(|_| default.to_string());
    print!("{} [{}]: ", label, current);
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return current;
    }

    let trimmed = input.trim();
    if trimmed.is_empty() {
        current
    } else {
        trimmed.to_string()
    }
}

fn prompt_required(label: &str, env_key: &str) -> String {
    loop {
        let current = env::var(env_key).unwrap_or_default();
        if current.is_empty() {
            print!("{}: ", label);
        } else {
            print!("{} [{}]: ", label, current);
        }
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            if !current.is_empty() {
                return current;
            }
            continue;
        }

        let trimmed = input.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
        if !current.is_empty() {
            return current;
        }

        println!("This field is required.");
    }
}

fn run_agent(vars: &[(&str, String)]) -> anyhow::Result<()> {
    let mut cmd = Command::new("nodeunion-agent");
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    for (k, v) in vars {
        cmd.env(k, v);
    }

    match cmd.status() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("nodeunion-agent exited with status {}", status))
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            let mut fallback = Command::new("cargo");
            fallback
                .arg("run")
                .arg("-p")
                .arg("nodeunion-agent")
                .arg("--bin")
                .arg("nodeunion-agent")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());

            for (k, v) in vars {
                fallback.env(k, v);
            }

            let status = fallback.status()?;
            if status.success() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("fallback cargo run exited with status {}", status))
            }
        }
        Err(err) => Err(err.into()),
    }
}

fn main() -> anyhow::Result<()> {
    println!("NodeUnion Agent Launch Form");
    println!("Fill values and press enter to keep defaults where available.");
    println!();

    let node_id = prompt_required("NODE_ID", "NODE_ID");
    let bind_addr = prompt("AGENT_BIND_ADDR", "AGENT_BIND_ADDR", "0.0.0.0:8090");
    let orchestrator_base_url = prompt_required("ORCHESTRATOR_BASE_URL", "ORCHESTRATOR_BASE_URL");
    let heartbeat_interval_secs = prompt("HEARTBEAT_INTERVAL_SECS", "HEARTBEAT_INTERVAL_SECS", "60");
    let metrics_poll_interval_secs = prompt("METRICS_POLL_INTERVAL_SECS", "METRICS_POLL_INTERVAL_SECS", "30");
    let idle_cpu_threshold_pct = prompt("IDLE_CPU_THRESHOLD_PCT", "IDLE_CPU_THRESHOLD_PCT", "15.0");
    let preempt_cpu_threshold_pct = prompt("PREEMPT_CPU_THRESHOLD_PCT", "PREEMPT_CPU_THRESHOLD_PCT", "60.0");
    let idle_window_samples = prompt("IDLE_WINDOW_SAMPLES", "IDLE_WINDOW_SAMPLES", "10");
    let request_timeout_secs = prompt("REQUEST_TIMEOUT_SECS", "REQUEST_TIMEOUT_SECS", "10");

    let vars = vec![
        ("NODE_ID", node_id),
        ("AGENT_BIND_ADDR", bind_addr),
        ("ORCHESTRATOR_BASE_URL", orchestrator_base_url),
        ("HEARTBEAT_INTERVAL_SECS", heartbeat_interval_secs),
        ("METRICS_POLL_INTERVAL_SECS", metrics_poll_interval_secs),
        ("IDLE_CPU_THRESHOLD_PCT", idle_cpu_threshold_pct),
        ("PREEMPT_CPU_THRESHOLD_PCT", preempt_cpu_threshold_pct),
        ("IDLE_WINDOW_SAMPLES", idle_window_samples),
        ("REQUEST_TIMEOUT_SECS", request_timeout_secs),
    ];

    println!();
    println!("Starting agent...");
    run_agent(&vars)
}
