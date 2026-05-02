use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::env;
use nodeunion_orchestrator::model::{
    JobRecord, JobStatus, NetworkRecord, NodeRecord, StopJobRequest, StopJobResponse,
    SubmitJobRequest, SubmitJobResponse,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Row, Table, Tabs, Wrap},
    Terminal,
};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::io::{self, Stdout, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tokio::sync::mpsc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveTab {
    Portfolio,
    Deploy,
}

impl ActiveTab {
    fn all() -> [ActiveTab; 2] {
        [ActiveTab::Portfolio, ActiveTab::Deploy]
    }

    fn title(self) -> &'static str {
        match self {
            ActiveTab::Portfolio => "Portfolio",
            ActiveTab::Deploy => "Deploy New Project",
        }
    }

    fn index(self) -> usize {
        match self {
            ActiveTab::Portfolio => 0,
            ActiveTab::Deploy => 1,
        }
    }

    fn from_index(index: usize) -> Self {
        match index {
            1 => ActiveTab::Deploy,
            _ => ActiveTab::Portfolio,
        }
    }
}

#[derive(Clone, Debug)]
struct Snapshot {
    healthy: bool,
    networks: Vec<NetworkRecord>,
    nodes: Vec<NodeRecord>,
    jobs: Vec<JobRecord>,
    errors: Vec<String>,
    fetched_at_epoch: u64,
}

#[derive(Clone, Debug)]
struct DeployResult {
    job_id: String,
    network_name: String,
    network_id: String,
    status: JobStatus,
    assigned_node_id: Option<String>,
    deploy_url: Option<String>,
    message: String,
    error_detail: Option<String>,
}

#[derive(Clone, Debug)]
struct DeployForm {
    orchestrator_url: String,
    selected_network: usize,
    network_query: String,
    wallet_address: String,
    dockerfile_path: String,
    context_path: String,
    image_tag: String,
    command_raw: String,
    cpu_limit: String,
    ram_limit_mb: String,
    exposed_port: String,
    push_image: bool,
    focus: usize,
    status_message: String,
    last_result: Option<DeployResult>,
}

impl DeployForm {
    fn new(orchestrator_url: String, networks: &[NetworkRecord]) -> Self {
        let default_context = "".to_string();
        Self {
            orchestrator_url,
            selected_network: 0.min(networks.len().saturating_sub(1)),
            network_query: String::new(),
            wallet_address: String::new(),
            dockerfile_path: String::new(),
            context_path: default_context,
            image_tag: "nodeunion-user-job:latest".to_string(),
            command_raw: String::new(),
            cpu_limit: "0.25".to_string(),
            ram_limit_mb: "128".to_string(),
            exposed_port: "3000".to_string(),
            push_image: true,
            focus: 0,
            status_message: "Fill the form, then press Enter on Submit.".to_string(),
            last_result: None,
        }
    }

    fn network<'a>(&self, networks: &'a [NetworkRecord]) -> Option<&'a NetworkRecord> {
        networks.get(self.selected_network)
    }

    fn focus_count() -> usize {
        12
    }

    fn move_focus_next(&mut self) {
        self.focus = (self.focus + 1) % Self::focus_count();
    }

    fn move_focus_prev(&mut self) {
        if self.focus == 0 {
            self.focus = Self::focus_count() - 1;
        } else {
            self.focus -= 1;
        }
    }

    fn current_text_mut(&mut self) -> Option<&mut String> {
        match self.focus {
            0 => Some(&mut self.orchestrator_url),
            2 => Some(&mut self.wallet_address),
            3 => Some(&mut self.dockerfile_path),
            4 => Some(&mut self.context_path),
            5 => Some(&mut self.image_tag),
            6 => Some(&mut self.command_raw),
            7 => Some(&mut self.cpu_limit),
            8 => Some(&mut self.ram_limit_mb),
            9 => Some(&mut self.exposed_port),
            _ => None,
        }
    }

    fn current_field_name(&self) -> &'static str {
        match self.focus {
            0 => "Orchestrator URL",
            1 => "Network",
            2 => "Wallet",
            3 => "Dockerfile",
            4 => "Context",
            5 => "Image tag",
            6 => "Command",
            7 => "CPU limit",
            8 => "RAM limit",
            9 => "Exposed port",
            10 => "Push image",
            _ => "Submit",
        }
    }

    fn insert_char(&mut self, ch: char) {
        if let Some(text) = self.current_text_mut() {
            text.push(ch);
        }
    }

    fn insert_text(&mut self, value: &str) {
        if let Some(text) = self.current_text_mut() {
            text.push_str(value);
        }
    }

    fn backspace(&mut self) {
        if let Some(text) = self.current_text_mut() {
            text.pop();
        }
    }

    fn toggle_push_image(&mut self) {
        self.push_image = !self.push_image;
    }

    fn cycle_network(&mut self, delta: i32, network_len: usize) {
        if network_len == 0 {
            self.selected_network = 0;
            return;
        }

        let current = self.selected_network as i32;
        let next = (current + delta).rem_euclid(network_len as i32);
        self.selected_network = next as usize;
        self.network_query.clear();
    }

    fn push_network_query_char(&mut self, ch: char, networks: &[NetworkRecord]) {
        self.network_query.push(ch);
        self.apply_network_query(networks);
    }

    fn pop_network_query_char(&mut self, networks: &[NetworkRecord]) {
        self.network_query.pop();
        self.apply_network_query(networks);
    }

    fn set_network_query(&mut self, value: &str, networks: &[NetworkRecord]) {
        self.network_query = value.to_string();
        self.apply_network_query(networks);
    }

    fn apply_network_query(&mut self, networks: &[NetworkRecord]) {
        if networks.is_empty() {
            self.selected_network = 0;
            return;
        }

        let needle = self.network_query.trim().to_lowercase();
        if needle.is_empty() {
            return;
        }

        if let Some((index, _)) = networks.iter().enumerate().find(|(_, network)| {
            network.name.to_lowercase().contains(&needle)
                || network.network_id.to_lowercase().contains(&needle)
        }) {
            self.selected_network = index;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EventOutcome {
    Continue,
    Refresh,
    Quit,
}

#[derive(Clone, Debug)]
enum SubmissionMessage {
    StatusUpdate(String),
    Success(DeployResult),
    Error(String),
    StopSuccess(String),
    StopError(String),
}

fn sorted_jobs(snapshot: &Snapshot) -> Vec<JobRecord> {
    let mut jobs = snapshot.jobs.clone();
    jobs.sort_by_key(|job| Reverse(job.created_at_epoch_secs));
    jobs
}

fn selected_job_id(snapshot: &Snapshot, selected: usize) -> Option<String> {
    let jobs = sorted_jobs(snapshot);
    jobs.get(selected)
        .or_else(|| jobs.first())
        .map(|job| job.job_id.clone())
}

fn launch_stop_job(
    snapshot: &Snapshot,
    selected: usize,
    client: &Client,
    base_url: &str,
    tx: &mpsc::Sender<SubmissionMessage>,
) {
    let Some(job_id) = selected_job_id(snapshot, selected) else {
        return;
    };

    let client_clone = client.clone();
    let base_url = base_url.trim_end_matches('/').to_string();
    let tx_clone = tx.clone();

    tokio::spawn(async move {
        let _ = tx_clone
            .send(SubmissionMessage::StatusUpdate(format!(
                "Stopping job {} ...",
                job_id
            )))
            .await;

        let req = StopJobRequest {
            reason: Some("stopped by user from TUI".to_string()),
        };

        let response = client_clone
            .post(format!("{}/jobs/{}/stop", base_url, job_id))
            .json(&req)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<StopJobResponse>().await {
                    Ok(stop_resp) => {
                        let _ = tx_clone
                            .send(SubmissionMessage::StopSuccess(format!(
                                "{} ({})",
                                stop_resp.message, stop_resp.job_id
                            )))
                            .await;
                    }
                    Err(err) => {
                        let _ = tx_clone
                            .send(SubmissionMessage::StopError(format!(
                                "stop succeeded but response decode failed: {}",
                                err
                            )))
                            .await;
                    }
                }
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "<unreadable response body>".to_string());
                let _ = tx_clone
                    .send(SubmissionMessage::StopError(format!(
                        "stop failed ({}): {}",
                        status,
                        sanitize_for_tui(&body)
                    )))
                    .await;
            }
            Err(err) => {
                let _ = tx_clone
                    .send(SubmissionMessage::StopError(format!(
                        "stop request failed: {}",
                        err
                    )))
                    .await;
            }
        }
    });
}

fn prompt(label: &str, default: &str) -> String {
    print!("{} [{}]: ", label, default);
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return default.to_string();
    }

    let trimmed = input.trim();
    if trimmed.is_empty() {
        default.to_string()
    } else {
        trimmed.to_string()
    }
}

async fn fetch_json<T: DeserializeOwned>(client: &Client, url: &str) -> anyhow::Result<T> {
    let response = client.get(url).send().await?;
    let status = response.status();

    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable>".to_string());
        anyhow::bail!("GET {} failed with {}: {}", url, status, body);
    }

    Ok(response.json::<T>().await?)
}

fn short_text(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let shortened: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{}…", shortened)
    } else {
        shortened
    }
}

fn parse_command(input: &str) -> Option<Vec<String>> {
    let parts = input
        .split(',')
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        None
    } else {
        Some(parts)
    }
}

fn resolve_build_context(dockerfile_abs: &str, context_input: &str) -> anyhow::Result<String> {
    let dockerfile_parent = Path::new(dockerfile_abs)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let raw_context = context_input.trim();
    let normalized = if raw_context.is_empty() || raw_context == "/" || raw_context == "." {
        dockerfile_parent
    } else if Path::new(raw_context).is_absolute() {
        PathBuf::from(raw_context)
    } else {
        let cwd = std::env::current_dir()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| PathBuf::from("."));
        cwd.join(raw_context)
    };

    if normalized.as_path() == Path::new("/") {
        anyhow::bail!(
            "Build context '/' is not allowed. Leave Context blank or set it to the Dockerfile folder."
        );
    }

    let metadata = std::fs::metadata(&normalized)
        .map_err(|_| anyhow::anyhow!("Build context not found: {}", normalized.display()))?;
    if !metadata.is_dir() {
        anyhow::bail!("Build context must be a directory: {}", normalized.display());
    }

    Ok(normalized.to_string_lossy().to_string())
}

fn normalize_image_tag(raw: &str) -> anyhow::Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        anyhow::bail!("Image tag is required");
    }

    if trimmed.split_whitespace().nth(1).is_some() {
        anyhow::bail!("Image tag cannot contain spaces; use a single tag like owner/name:tag");
    }

    Ok(trimmed.to_string())
}

fn node_label(network_id: &str, networks: &[NetworkRecord]) -> String {
    networks
        .iter()
        .find(|network| network.network_id == network_id)
        .map(|network| format!("{} ({})", network.name, network.network_id))
        .unwrap_or_else(|| network_id.to_string())
}

fn network_node_count(network_id: &str, nodes: &[NodeRecord]) -> usize {
    nodes.iter().filter(|node| node.network_id == network_id).count()
}

fn node_status_style(status: &nodeunion_orchestrator::model::NodeStatus) -> Style {
    match status {
        nodeunion_orchestrator::model::NodeStatus::Idle => Style::default().fg(Color::Green),
        nodeunion_orchestrator::model::NodeStatus::Busy => Style::default().fg(Color::Yellow),
        nodeunion_orchestrator::model::NodeStatus::Draining => Style::default().fg(Color::Magenta),
        nodeunion_orchestrator::model::NodeStatus::Preempting => Style::default().fg(Color::Red),
        nodeunion_orchestrator::model::NodeStatus::Offline => Style::default().fg(Color::DarkGray),
    }
}

fn job_status_style(status: &JobStatus) -> Style {
    match status {
        JobStatus::Pending => Style::default().fg(Color::Yellow),
        JobStatus::Scheduled => Style::default().fg(Color::Cyan),
        JobStatus::Running => Style::default().fg(Color::Green),
        JobStatus::Done => Style::default().fg(Color::Blue),
        JobStatus::Failed => Style::default().fg(Color::Red),
        JobStatus::Preempted => Style::default().fg(Color::Magenta),
        JobStatus::Stopped => Style::default().fg(Color::DarkGray),
    }
}

fn job_status_label(status: &JobStatus) -> &'static str {
    match status {
        JobStatus::Pending => "PENDING",
        JobStatus::Scheduled => "SCHEDULED",
        JobStatus::Running => "RUNNING",
        JobStatus::Done => "DONE",
        JobStatus::Failed => "FAILED",
        JobStatus::Preempted => "PREEMPTED",
        JobStatus::Stopped => "STOPPED",
    }
}

fn percent_ratio(value: f64) -> f64 {
    (value / 100.0).clamp(0.0, 1.0)
}

fn node_age_secs(node: &NodeRecord, now: u64) -> u64 {
    now.saturating_sub(node.last_seen_epoch_secs)
}

fn sanitize_for_tui(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_control() && ch != '\n' && ch != '\t' {
                ' '
            } else {
                ch
            }
        })
        .collect()
}

fn tail_lines(text: &str, max_lines: usize) -> String {
    let mut lines = text.lines().collect::<Vec<_>>();
    if lines.len() > max_lines {
        lines = lines.split_off(lines.len() - max_lines);
    }
    lines.join("\n")
}

fn sync_last_result_with_snapshot(form: &mut DeployForm, snapshot: &Snapshot) {
    let Some(last_result) = form.last_result.as_mut() else {
        return;
    };

    if last_result.job_id == "-" {
        return;
    }

    let Some(job) = snapshot.jobs.iter().find(|job| job.job_id == last_result.job_id) else {
        return;
    };

    last_result.status = job.status.clone();
    last_result.assigned_node_id = job.assigned_node_id.clone();
    last_result.deploy_url = job.deploy_url.clone();
    last_result.error_detail = job.error_detail.clone();
    last_result.message = match job.status {
        JobStatus::Pending => "job accepted, waiting for idle node".to_string(),
        JobStatus::Scheduled | JobStatus::Running => "job dispatched to node".to_string(),
        JobStatus::Done => "job completed".to_string(),
        JobStatus::Stopped => "job stopped".to_string(),
        JobStatus::Failed => "job failed".to_string(),
        JobStatus::Preempted => "job preempted".to_string(),
    };

    if let Some(url) = &last_result.deploy_url {
        form.status_message = format!("✓ Deployed: {}", url);
    } else if matches!(job.status, JobStatus::Running | JobStatus::Scheduled) {
        form.status_message = "Job is running; public URL is still pending from node tunnel setup.".to_string();
    }
}

fn effective_job_url(job: &JobRecord, _nodes: &[NodeRecord]) -> Option<String> {
    if let Some(url) = &job.deploy_url {
        return Some(url.clone());
    }

    None
}

fn run_builder(command: &str, args: &[String], phase: &str) -> anyhow::Result<()> {
    let mut process = Command::new(command);
    process.args(args);
    let output = process.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let raw = if stderr.trim().is_empty() { stdout } else { stderr };
        let details = tail_lines(&sanitize_for_tui(&raw), 12);

        if details.trim().is_empty() {
            anyhow::bail!("{} failed during {} with status {}", command, phase, output.status);
        } else {
            anyhow::bail!("{} failed during {} with status {}:\n{}", command, phase, output.status, details);
        }
    }
    Ok(())
}

async fn docker_build(dockerfile_path: &str, context_path: &str, image_tag: &str) -> anyhow::Result<()> {
    println!("\nBuilding docker image {} ...", image_tag);
    let dockerfile_path = dockerfile_path.to_string();
    let context_path = context_path.to_string();
    let image_tag = image_tag.to_string();
    
    tokio::task::spawn_blocking(move || {
        run_builder(
            "docker",
            &[
                "build".to_string(),
                "--progress=plain".to_string(),
                "-f".to_string(),
                dockerfile_path,
                "-t".to_string(),
                image_tag,
                context_path,
            ],
            "build",
        )
    })
    .await
    .map_err(|e| anyhow::anyhow!("spawn_blocking error: {}", e))?
}

async fn docker_push(image_tag: &str) -> anyhow::Result<()> {
    println!("\nPushing docker image {} ...", image_tag);
    let image_tag = image_tag.to_string();
    
    tokio::task::spawn_blocking(move || {
        run_builder("docker", &["push".to_string(), image_tag], "push")
    })
    .await
    .map_err(|e| anyhow::anyhow!("spawn_blocking error: {}", e))?
}

fn collect_snapshot(
    healthy: bool,
    networks: Vec<NetworkRecord>,
    nodes: Vec<NodeRecord>,
    jobs: Vec<JobRecord>,
    errors: Vec<String>,
) -> Snapshot {
    let fetched_at_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    Snapshot {
        healthy,
        networks,
        nodes,
        jobs,
        errors,
        fetched_at_epoch,
    }
}

async fn refresh_snapshot(client: &Client, base_url: &str) -> Snapshot {
    let mut errors = Vec::new();

    // Pre-compute URLs to avoid temporary value issues
    let health_url = format!("{}/health", base_url);
    let networks_url = format!("{}/networks", base_url);
    let nodes_url = format!("{}/nodes", base_url);
    let jobs_url = format!("{}/jobs", base_url);

    // Parallelize all HTTP requests
    let health_future = client.get(&health_url).send();
    let networks_future = fetch_json::<Vec<NetworkRecord>>(client, &networks_url);
    let nodes_future = fetch_json::<Vec<NodeRecord>>(client, &nodes_url);
    let jobs_future = fetch_json::<Vec<JobRecord>>(client, &jobs_url);

    let (health_res, networks_res, nodes_res, jobs_res) = tokio::join!(
        health_future,
        networks_future,
        nodes_future,
        jobs_future,
    );

    let healthy = match health_res {
        Ok(resp) => resp.status().is_success(),
        Err(err) => {
            errors.push(format!("health check failed: {}", err));
            false
        }
    };

    let networks = match networks_res {
        Ok(data) => data,
        Err(err) => {
            errors.push(err.to_string());
            Vec::new()
        }
    };

    let nodes = match nodes_res {
        Ok(data) => data,
        Err(err) => {
            errors.push(err.to_string());
            Vec::new()
        }
    };

    let jobs = match jobs_res {
        Ok(data) => data,
        Err(err) => {
            errors.push(err.to_string());
            Vec::new()
        }
    };

    collect_snapshot(healthy, networks, nodes, jobs, errors)
}

async fn poll_deploy_url(
    client: &Client,
    base_url: &str,
    job_id: &str,
    timeout_secs: u64,
) -> anyhow::Result<Option<String>> {
    let deadline = SystemTime::now() + Duration::from_secs(timeout_secs);

    loop {
        let jobs = fetch_json::<Vec<JobRecord>>(client, &format!("{}/jobs", base_url)).await?;
        if let Some(job) = jobs.into_iter().find(|job| job.job_id == job_id) {
            if job.deploy_url.is_some() {
                return Ok(job.deploy_url);
            }
        }

        if SystemTime::now() >= deadline {
            return Ok(None);
        }

        sleep(Duration::from_secs(2)).await;
    }
}

async fn submit_deploy_in_background(
    client: Client,
    snapshot: Snapshot,
    form_data: (String, usize, String, String, String, String, String, String, String, String, bool),
    _orchestrator_url: String,
    tx: mpsc::Sender<SubmissionMessage>,
) {
    let (orchestra_url, selected_network_idx, wallet, dockerfile, context, image_tag, command, cpu, ram, port, push) = form_data;
    
    let result = async {
        let selected_network = snapshot
            .networks
            .get(selected_network_idx)
            .ok_or_else(|| anyhow::anyhow!("no network selected"))?;

        if wallet.trim().is_empty() {
            anyhow::bail!("Wallet is required (user_wallet cannot be empty)");
        }

        let image_tag = normalize_image_tag(&image_tag)?;

        let dockerfile_path = dockerfile.trim();
        if dockerfile_path.is_empty() {
            anyhow::bail!("Dockerfile path is required");
        }
        
        // Resolve relative paths to absolute paths
        let dockerfile_abs = if Path::new(dockerfile_path).is_absolute() {
            dockerfile_path.to_string()
        } else {
            let cwd = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string());
            PathBuf::from(&cwd)
                .join(dockerfile_path)
                .to_string_lossy()
                .to_string()
        };
        
        if !Path::new(&dockerfile_abs).exists() {
            anyhow::bail!(
                "Dockerfile not found at: {} (relative to: {})",
                dockerfile_abs,
                std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "(unknown working directory)".to_string())
            );
        }

        let context_path = resolve_build_context(&dockerfile_abs, &context)?;

        let cpu_limit = cpu.trim().parse::<f64>().unwrap_or(0.25);
        let ram_limit_mb = ram.trim().parse::<u64>().unwrap_or(128);
        let exposed_port = port
            .trim()
            .parse::<u16>()
            .ok()
            .filter(|p| *p > 0);
        let parsed_command = parse_command(&command);

        let _ = tx.send(SubmissionMessage::StatusUpdate(format!("Building image {} using context {} ...", image_tag, context_path))).await;
        docker_build(&dockerfile_abs, &context_path, &image_tag).await?;

        if push {
            let _ = tx.send(SubmissionMessage::StatusUpdate(format!("Pushing image {} ...", image_tag))).await;
            docker_push(&image_tag).await?;
        }

        let _ = tx.send(SubmissionMessage::StatusUpdate("Submitting job ...".to_string())).await;
        let payload = SubmitJobRequest {
            network_id: selected_network.network_id.clone(),
            user_wallet: wallet.trim().to_string(),
            image: image_tag,
            command: parsed_command,
            cpu_limit,
            ram_limit_mb,
            exposed_port,
        };

        let response = client
            .post(format!("{}/jobs/submit", orchestra_url))
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<unreadable response body>".to_string());

            let api_error = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|value| value.get("error").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .unwrap_or_else(|| body.clone());

            anyhow::bail!("submit failed ({}): {}", status, sanitize_for_tui(&api_error));
        }

        let submit_response: SubmitJobResponse = response.json().await?;
        let mut deploy_url = submit_response.deploy_url.clone();

        if deploy_url.is_none() && exposed_port.is_some() {
            let _ = tx.send(SubmissionMessage::StatusUpdate("Waiting for the public URL to appear ...".to_string())).await;
            deploy_url = poll_deploy_url(&client, &orchestra_url, &submit_response.job_id, 120).await?;
        }

        Ok::<DeployResult, anyhow::Error>(DeployResult {
            job_id: submit_response.job_id,
            network_name: selected_network.name.clone(),
            network_id: selected_network.network_id.clone(),
            status: submit_response.status,
            assigned_node_id: submit_response.assigned_node_id,
            deploy_url,
            message: submit_response.message,
            error_detail: None,
        })
    }.await;

    match result {
        Ok(deploy_result) => {
            let _ = tx.send(SubmissionMessage::Success(deploy_result)).await;
        }
        Err(err) => {
            // Include full error chain for debugging
            let mut error_msg = format!("{}", err);
            let mut source = err.source();
            while let Some(s) = source {
                error_msg.push_str(&format!("\n  Caused by: {}", s));
                source = s.source();
            }
            let _ = tx.send(SubmissionMessage::Error(error_msg)).await;
        }
    }
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn should_exit() -> io::Result<bool> {
    if event::poll(Duration::from_millis(1))? {
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                return Ok(true);
            }
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn render_tabs(area: Rect, frame: &mut Frame<'_>, active: ActiveTab) {
    let titles = ActiveTab::all()
        .iter()
        .map(|tab| Line::from(Span::styled(tab.title(), Style::default().fg(Color::White))))
        .collect::<Vec<_>>();

    let tabs = Tabs::new(titles)
        .select(active.index())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    "NodeUnion User Portal",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

fn render_header(area: Rect, frame: &mut Frame<'_>, snapshot: &Snapshot, base_url: &str) {
    let health_color = if snapshot.healthy { Color::Green } else { Color::Red };
    let status_text = if snapshot.healthy { "ONLINE" } else { "OFFLINE" };

    let title = Line::from(vec![
        Span::styled(
            "Deploy Portal ",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::styled("● ", Style::default().fg(health_color).add_modifier(Modifier::BOLD)),
        Span::styled(status_text, Style::default().fg(health_color).add_modifier(Modifier::BOLD)),
    ]);

    let body = vec![
        Line::from(vec![
            Span::styled("orchestrator ", Style::default().fg(Color::DarkGray)),
            Span::raw(base_url.to_string()),
            Span::styled("   refreshed ", Style::default().fg(Color::DarkGray)),
            Span::raw(snapshot.fetched_at_epoch.to_string()),
        ]),
        Line::from(vec![
            Span::styled("keys ", Style::default().fg(Color::DarkGray)),
            Span::raw("Tab switch tabs  F1 portfolio  F2 deploy  Enter/F5/Ctrl+S submit  q/Ctrl+C quit"),
        ]),
        Line::from(vec![
            Span::styled("networks ", Style::default().fg(Color::DarkGray)),
            Span::raw(snapshot.networks.len().to_string()),
            Span::styled("   nodes ", Style::default().fg(Color::DarkGray)),
            Span::raw(snapshot.nodes.len().to_string()),
            Span::styled("   jobs ", Style::default().fg(Color::DarkGray)),
            Span::raw(snapshot.jobs.len().to_string()),
        ]),
    ];

    let paragraph = Paragraph::new(Text::from(body))
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn render_summary(area: Rect, frame: &mut Frame<'_>, snapshot: &Snapshot) {
    let total_nodes = snapshot.nodes.len();
    let idle_nodes = snapshot.nodes.iter().filter(|node| node.is_idle).count();
    let busy_nodes = total_nodes.saturating_sub(idle_nodes);
    let running_jobs = snapshot
        .jobs
        .iter()
        .filter(|job| matches!(job.status, JobStatus::Running | JobStatus::Scheduled))
        .count();
    let failed_jobs = snapshot
        .jobs
        .iter()
        .filter(|job| matches!(job.status, JobStatus::Failed))
        .count();
    let deployed_jobs = snapshot.jobs.iter().filter(|job| job.deploy_url.is_some()).count();
    let avg_cpu_available = if total_nodes == 0 {
        0.0
    } else {
        snapshot
            .nodes
            .iter()
            .map(|node| node.cpu_available_pct as f64)
            .sum::<f64>()
            / total_nodes as f64
    };

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(22),
            Constraint::Percentage(22),
            Constraint::Percentage(22),
            Constraint::Percentage(34),
        ])
        .split(area);

    let cloud_gauge = Paragraph::new(Text::from(vec![
        Line::from(vec![Span::styled(
            format!("{} busy / {} total", busy_nodes, total_nodes),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            format!("{} idle nodes", idle_nodes),
            Style::default().fg(Color::DarkGray),
        )]),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Cloud Load"));

    let jobs_card = Paragraph::new(Text::from(vec![
        Line::from(vec![Span::styled(
            format!("{} active / {} total", running_jobs, snapshot.jobs.len()),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            format!("{} failed", failed_jobs),
            Style::default().fg(Color::Red),
        )]),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Job Activity"));

    let cpu_card = Paragraph::new(Text::from(vec![
        Line::from(vec![Span::styled(
            format!("{:.1}% free", avg_cpu_available),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            format!("{} deployed jobs", deployed_jobs),
            Style::default().fg(Color::DarkGray),
        )]),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Capacity"));

    let networks_card = Paragraph::new(Text::from(vec![
        Line::from(vec![Span::styled(
            snapshot.networks.len().to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "active networks and deployments",
            Style::default().fg(Color::DarkGray),
        )]),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Portfolio"));

    frame.render_widget(cloud_gauge, columns[0]);
    frame.render_widget(jobs_card, columns[1]);
    frame.render_widget(cpu_card, columns[2]);
    frame.render_widget(networks_card, columns[3]);
}

fn render_portfolio(area: Rect, frame: &mut Frame<'_>, snapshot: &Snapshot, selected: usize) {
    let jobs = sorted_jobs(snapshot);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    let summary_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(12), Constraint::Length(9)])
        .split(area);

    render_summary(summary_area[0], frame, snapshot);

    let rows = if jobs.is_empty() {
        vec![Row::new(vec![
            "No jobs found",
            "-",
            "-",
            "-",
            "-",
            "-",
        ])]
    } else {
        jobs
            .iter()
            .enumerate()
            .map(|(index, job)| {
                let network_name = node_label(&job.network_id, &snapshot.networks);
                let deploy_url = effective_job_url(job, &snapshot.nodes)
                    .as_deref()
                    .map(|url| short_text(url, 30))
                    .unwrap_or_else(|| "-".to_string());
                let error_text = job
                    .error_detail
                    .as_deref()
                    .map(|text| short_text(text, 36))
                    .unwrap_or_else(|| "-".to_string());
                let mut row = Row::new(vec![
                    short_text(&job.job_id, 18),
                    short_text(&network_name, 18),
                    job_status_label(&job.status).to_string(),
                    job.assigned_node_id.clone().unwrap_or_else(|| "-".to_string()),
                    deploy_url,
                    error_text,
                ]);

                if index == selected.min(jobs.len().saturating_sub(1)) {
                    row = row.style(Style::default().add_modifier(Modifier::REVERSED));
                } else {
                    row = row.style(job_status_style(&job.status));
                }

                row
            })
            .collect()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(18),
            Constraint::Length(18),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Length(32),
            Constraint::Min(20),
        ],
    )
    .header(
        Row::new(vec!["Job", "Network", "Status", "Node", "Deploy URL", "Error"])
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled("Portfolio", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
    )
    .column_spacing(1);

    frame.render_widget(table, summary_area[1]);

    let selected_job = jobs.get(selected).or_else(|| jobs.first());
    let details = if let Some(job) = selected_job {
        let network_name = node_label(&job.network_id, &snapshot.networks);
        let mut lines = vec![
            Line::from(vec![
                Span::styled("job ", Style::default().fg(Color::DarkGray)),
                Span::raw(job.job_id.clone()),
                Span::styled("   network ", Style::default().fg(Color::DarkGray)),
                Span::raw(network_name),
            ]),
            Line::from(vec![
                Span::styled("status ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    job_status_label(&job.status),
                    job_status_style(&job.status).add_modifier(Modifier::BOLD),
                ),
                Span::styled("   node ", Style::default().fg(Color::DarkGray)),
                Span::raw(job.assigned_node_id.clone().unwrap_or_else(|| "-".to_string())),
            ]),
            Line::from(vec![
                Span::styled("url ", Style::default().fg(Color::DarkGray)),
                Span::raw(
                    effective_job_url(job, &snapshot.nodes)
                        .unwrap_or_else(|| "pending".to_string()),
                ),
            ]),
            Line::from(vec![
                Span::styled("error ", Style::default().fg(Color::DarkGray)),
                Span::raw(job.error_detail.clone().unwrap_or_else(|| "none".to_string())),
            ]),
            Line::from(vec![
                Span::styled("created ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{}", job.created_at_epoch_secs)),
                Span::styled("   age ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{}s", node_age_secs(&snapshot.nodes.iter().find(|node| node.node_id == job.assigned_node_id.clone().unwrap_or_default()).cloned().unwrap_or_else(|| NodeRecord {
                    node_id: String::new(),
                    network_id: String::new(),
                    agent_url: String::new(),
                    provider_wallet: None,
                    region: None,
                    labels: HashMap::new(),
                    status: nodeunion_orchestrator::model::NodeStatus::Offline,
                    is_idle: true,
                    cpu_available_pct: 0.0,
                    ram_available_mb: 0,
                    disk_available_gb: 0,
                    running_chunks: 0,
                    last_seen_epoch_secs: now,
                }), now))),
            ]),
        ];

        if let Some(command) = &job.command {
            lines.push(Line::from(vec![
                Span::styled("command ", Style::default().fg(Color::DarkGray)),
                Span::raw(command.join(", ")),
            ]));
        }

        Text::from(lines)
    } else {
        Text::from("No job selected")
    };

    let details_widget = Paragraph::new(details)
        .block(Block::default().borders(Borders::ALL).title("Selected Job (x: stop)"))
        .wrap(Wrap { trim: true });

    frame.render_widget(details_widget, summary_area[2]);
}

fn render_field(
    area: Rect,
    frame: &mut Frame<'_>,
    label: &str,
    value: String,
    active: bool,
    hint: &str,
) {
    let border = if active { Color::Cyan } else { Color::DarkGray };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(Span::styled(label, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));

    let text = vec![
        Line::from(value),
        Line::from(vec![Span::styled(
            hint,
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    frame.render_widget(Paragraph::new(Text::from(text)).block(block).wrap(Wrap { trim: true }), area);
}

fn render_deploy(area: Rect, frame: &mut Frame<'_>, snapshot: &Snapshot, form: &DeployForm) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(20), Constraint::Length(8)])
        .split(area);

    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "(unknown)".to_string());
    
    let status_card = Paragraph::new(Text::from(vec![
        Line::from(vec![Span::styled(
            form.status_message.clone(),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            format!("Current field: {}", form.current_field_name()),
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(vec![Span::styled(
            format!("Working dir: {}", cwd),
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(vec![Span::styled(
            "Tip: Portfolio tab uses x to stop selected job. Deploy tab uses Tab/Shift+Tab, Left/Right, Enter on Submit.",
            Style::default().fg(Color::DarkGray),
        )]),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Deploy Status"))
    .wrap(Wrap { trim: true });

    frame.render_widget(status_card, layout[0]);

    let form_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[1]);

    let left_fields = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
        ])
        .split(form_cols[0]);

    let right_fields = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
        ])
        .split(form_cols[1]);

    render_field(
        left_fields[0],
        frame,
        "Orchestrator URL",
        form.orchestrator_url.clone(),
        form.focus == 0,
        "Base URL used to submit and poll jobs",
    );
    let network_label = form
        .network(&snapshot.networks)
        .map(|network| format!("{} ({}) - {} node(s)", network.name, network.network_id, network_node_count(&network.network_id, &snapshot.nodes)))
        .unwrap_or_else(|| "No networks available".to_string());
    let network_value = if form.network_query.trim().is_empty() {
        network_label
    } else {
        format!("{}  [query: {}]", network_label, form.network_query)
    };
    render_field(
        left_fields[1],
        frame,
        "Network",
        network_value,
        form.focus == 1,
        "Type to filter by name/id; Left/Right also cycles",
    );
    render_field(
        left_fields[2],
        frame,
        "Wallet",
        form.wallet_address.clone(),
        form.focus == 2,
        "Billing wallet or any test string",
    );
    render_field(
        left_fields[3],
        frame,
        "Dockerfile",
        form.dockerfile_path.clone(),
        form.focus == 3,
        "Path to the Dockerfile you want to build",
    );
    render_field(
        left_fields[4],
        frame,
        "Context",
        form.context_path.clone(),
        form.focus == 4,
        "Build context path; blank uses Dockerfile parent",
    );
    render_field(
        left_fields[5],
        frame,
        "Image tag",
        form.image_tag.clone(),
        form.focus == 5,
        "Local tag that will be submitted to the network",
    );

    render_field(
        right_fields[0],
        frame,
        "Command",
        form.command_raw.clone(),
        form.focus == 6,
        "Comma-separated command override; blank keeps image default",
    );
    render_field(
        right_fields[1],
        frame,
        "CPU limit",
        form.cpu_limit.clone(),
        form.focus == 7,
        "Example: 0.25",
    );
    render_field(
        right_fields[2],
        frame,
        "RAM limit MB",
        form.ram_limit_mb.clone(),
        form.focus == 8,
        "Example: 128",
    );
    render_field(
        right_fields[3],
        frame,
        "Exposed port",
        form.exposed_port.clone(),
        form.focus == 9,
        "Set this for web apps so the deploy URL can be returned",
    );
    render_field(
        right_fields[4],
        frame,
        "Push image",
        if form.push_image { "yes".to_string() } else { "no".to_string() },
        form.focus == 10,
        "Disable only if the image is already reachable on the remote node",
    );
    render_field(
        right_fields[5],
        frame,
        "Submit",
        "Press Enter to build and deploy".to_string(),
        form.focus == 11,
        "After submit the deploy URL will appear here when available",
    );

    let result_text = if let Some(result) = &form.last_result {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("job id ", Style::default().fg(Color::DarkGray)),
                Span::raw(result.job_id.clone()),
                Span::styled("   status ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    job_status_label(&result.status),
                    job_status_style(&result.status).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("network ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{} ({})", result.network_name, result.network_id)),
                Span::styled("   node ", Style::default().fg(Color::DarkGray)),
                Span::raw(result.assigned_node_id.clone().unwrap_or_else(|| "-".to_string())),
            ]),
            Line::from(vec![
                Span::styled("url ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    result
                        .deploy_url
                        .clone()
                        .unwrap_or_else(|| "pending".to_string()),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("message ", Style::default().fg(Color::DarkGray)),
                Span::raw(result.message.clone()),
            ]),
        ];

        if let Some(error) = &result.error_detail {
            lines.push(Line::from(vec![
                Span::styled("error ", Style::default().fg(Color::DarkGray)),
                Span::styled(error.clone(), Style::default().fg(Color::Red)),
            ]));
        }

        Text::from(lines)
    } else {
        Text::from(vec![
            Line::from(vec![Span::styled(
                "No deployment yet.",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(vec![Span::styled(
                "The result card will show the public URL once the job starts.",
                Style::default().fg(Color::DarkGray),
            )]),
        ])
    };

    let result_card = Paragraph::new(result_text)
        .block(Block::default().borders(Borders::ALL).title("Deployment Result"))
        .wrap(Wrap { trim: true });

    frame.render_widget(result_card, layout[2]);
}

fn render_errors(area: Rect, frame: &mut Frame<'_>, errors: &[String]) {
    let block = Block::default().borders(Borders::ALL).title("Diagnostics");

    if errors.is_empty() {
        frame.render_widget(Paragraph::new("No errors reported.").block(block), area);
        return;
    }

    let body = errors
        .iter()
        .map(|error| Line::from(vec![Span::styled(error.clone(), Style::default().fg(Color::Red))]))
        .collect::<Vec<_>>();

    frame.render_widget(Paragraph::new(Text::from(body)).block(block).wrap(Wrap { trim: true }), area);
}

fn render_app(frame: &mut Frame<'_>, snapshot: &Snapshot, base_url: &str, active_tab: ActiveTab, portfolio_selected: usize, form: &DeployForm) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Min(10),
            Constraint::Length(7),
        ])
        .split(frame.area());

    render_tabs(outer[0], frame, active_tab);
    render_header(outer[1], frame, snapshot, base_url);

    match active_tab {
        ActiveTab::Portfolio => {
            render_portfolio(outer[2], frame, snapshot, portfolio_selected);
            render_errors(outer[3], frame, &snapshot.errors);
        }
        ActiveTab::Deploy => {
            render_deploy(outer[2], frame, snapshot, form);
            render_errors(outer[3], frame, &snapshot.errors);
        }
    }
}

async fn handle_key(
    key: KeyEvent,
    active_tab: &mut ActiveTab,
    portfolio_selected: &mut usize,
    snapshot: &Snapshot,
    form: &mut DeployForm,
    client: &Client,
    base_url: &mut String,
    submission_tx: mpsc::Sender<SubmissionMessage>,
) -> io::Result<EventOutcome> {
    let launch_submit = |form: &mut DeployForm,
                         snapshot: &Snapshot,
                         client: &Client,
                         base_url: &str,
                         submission_tx: &mpsc::Sender<SubmissionMessage>| {
        if form.wallet_address.trim().is_empty() {
            form.status_message = "Wallet is required before submit".to_string();
            return;
        }
        if form.dockerfile_path.trim().is_empty() {
            form.status_message = "Dockerfile path is required before submit".to_string();
            return;
        }
        if form.image_tag.trim().is_empty() {
            form.status_message = "Image tag is required before submit".to_string();
            return;
        }
        if snapshot.networks.is_empty() {
            form.status_message = "No networks available for submit".to_string();
            return;
        }

        form.status_message = "Submitting...".to_string();

        let client_clone = client.clone();
        let snapshot_clone = snapshot.clone();
        let orchestrator_url_clone = base_url.to_string();
        let form_data = (
            form.orchestrator_url.clone(),
            form.selected_network,
            form.wallet_address.clone(),
            form.dockerfile_path.clone(),
            form.context_path.clone(),
            form.image_tag.clone(),
            form.command_raw.clone(),
            form.cpu_limit.clone(),
            form.ram_limit_mb.clone(),
            form.exposed_port.clone(),
            form.push_image,
        );
        let tx_clone = submission_tx.clone();

        tokio::spawn(submit_deploy_in_background(
            client_clone,
            snapshot_clone,
            form_data,
            orchestrator_url_clone,
            tx_clone,
        ));
    };

    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Ok(EventOutcome::Quit);
    }

    if matches!(*active_tab, ActiveTab::Portfolio) {
        match key.code {
            KeyCode::Tab | KeyCode::BackTab => {
                *active_tab = ActiveTab::Deploy;
                return Ok(EventOutcome::Continue);
            }
            _ => {}
        }
    }

    match key.code {
        KeyCode::F(1) => {
            *active_tab = ActiveTab::Portfolio;
            return Ok(EventOutcome::Continue);
        }
        KeyCode::F(2) => {
            *active_tab = ActiveTab::Deploy;
            return Ok(EventOutcome::Continue);
        }
        KeyCode::Char('r') if !key.modifiers.contains(KeyModifiers::CONTROL) && matches!(*active_tab, ActiveTab::Portfolio) => {
            return Ok(EventOutcome::Refresh);
        }
        KeyCode::Char('q') if !key.modifiers.contains(KeyModifiers::CONTROL) && matches!(*active_tab, ActiveTab::Portfolio) => {
            return Ok(EventOutcome::Quit);
        }
        _ => {}
    }

    match active_tab {
        ActiveTab::Portfolio => {
            let job_count = snapshot.jobs.len().max(1);
            match key.code {
                KeyCode::Up => {
                    if *portfolio_selected == 0 {
                        *portfolio_selected = job_count - 1;
                    } else {
                        *portfolio_selected -= 1;
                    }
                }
                KeyCode::Down => {
                    *portfolio_selected = (*portfolio_selected + 1) % job_count;
                }
                KeyCode::Home => *portfolio_selected = 0,
                KeyCode::End => *portfolio_selected = job_count - 1,
                KeyCode::Char('x') => {
                    launch_stop_job(snapshot, *portfolio_selected, client, base_url, &submission_tx);
                }
                _ => {}
            }
        }
        ActiveTab::Deploy => match key.code {
            KeyCode::F(5) => {
                launch_submit(form, snapshot, client, base_url, &submission_tx);
            }
            KeyCode::Tab => {
                form.move_focus_next();
            }
            KeyCode::BackTab => {
                form.move_focus_prev();
            }
            KeyCode::Up => {
                if form.focus == 1 {
                    form.cycle_network(-1, snapshot.networks.len());
                } else {
                    form.move_focus_prev();
                }
            }
            KeyCode::Down => {
                if form.focus == 1 {
                    form.cycle_network(1, snapshot.networks.len());
                } else {
                    form.move_focus_next();
                }
            }
            KeyCode::Left => {
                if form.focus == 1 {
                    form.cycle_network(-1, snapshot.networks.len());
                } else if form.focus == 10 {
                    form.push_image = false;
                }
            }
            KeyCode::Right => {
                if form.focus == 1 {
                    form.cycle_network(1, snapshot.networks.len());
                } else if form.focus == 10 {
                    form.push_image = true;
                }
            }
            KeyCode::Char(' ') if form.focus == 10 => {
                form.toggle_push_image();
            }
            KeyCode::Backspace => {
                if form.focus == 1 {
                    form.pop_network_query_char(&snapshot.networks);
                } else {
                    form.backspace();
                    if form.focus == 0 {
                        *base_url = form.orchestrator_url.trim_end_matches('/').to_string();
                    }
                }
            }
            KeyCode::Enter => {
                if form.focus < 11 {
                    if form.focus == 10 {
                        form.toggle_push_image();
                    } else {
                        form.move_focus_next();
                    }
                } else {
                    launch_submit(form, snapshot, client, base_url, &submission_tx);
                }
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                launch_submit(form, snapshot, client, base_url, &submission_tx);
            }
            KeyCode::Esc => {
                *active_tab = ActiveTab::Portfolio;
            }
            KeyCode::Char(ch) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    if form.focus == 1 {
                        form.push_network_query_char(ch, &snapshot.networks);
                    } else {
                        form.insert_char(ch);
                        if form.focus == 0 {
                            *base_url = form.orchestrator_url.trim_end_matches('/').to_string();
                        }
                        if form.focus == 3 && form.context_path.trim().is_empty() {
                            let candidate = PathBuf::from(&form.dockerfile_path);
                            if let Some(parent) = candidate.parent() {
                                form.context_path = parent.to_string_lossy().to_string();
                            }
                        }
                    }
                }
            }
            _ => {}
        },
    }

    Ok(EventOutcome::Continue)
}

pub async fn run(base_url: String) -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;
    let client = Client::new();
    let mut orchestrator_url = base_url.trim_end_matches('/').to_string();
    let mut snapshot = refresh_snapshot(&client, &orchestrator_url).await;
    let mut active_tab = ActiveTab::Portfolio;
    let mut portfolio_selected = 0usize;
    let mut form = DeployForm::new(orchestrator_url.clone(), &snapshot.networks);
    
    // Create channel for submission status updates
    let (submission_tx, mut submission_rx) = mpsc::channel(100);
    
    // Create channel for background refresh updates
    let (refresh_tx, mut refresh_rx) = mpsc::channel::<Snapshot>(1);
    {
        let client = client.clone();
        let url = orchestrator_url.clone();
        let tx = refresh_tx.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(10)).await;
                let snap = refresh_snapshot(&client, &url).await;
                let _ = tx.send(snap).await;
            }
        });
    }

    let run_result: anyhow::Result<()> = loop {
        if form.selected_network >= snapshot.networks.len() && !snapshot.networks.is_empty() {
            form.selected_network = 0;
        }

        sync_last_result_with_snapshot(&mut form, &snapshot);

        terminal.draw(|frame| {
            render_app(
                frame,
                &snapshot,
                &orchestrator_url,
                active_tab,
                portfolio_selected,
                &form,
            )
        })?;

        if event::poll(Duration::from_millis(120))? {
            match event::read()? {
                Event::Key(key) => {
                    let outcome = handle_key(
                        key,
                        &mut active_tab,
                        &mut portfolio_selected,
                        &snapshot,
                        &mut form,
                        &client,
                        &mut orchestrator_url,
                        submission_tx.clone(),
                    )
                    .await?;

                    match outcome {
                        EventOutcome::Quit => break Ok(()),
                        EventOutcome::Refresh => {
                            snapshot = refresh_snapshot(&client, &orchestrator_url).await;
                            terminal.draw(|frame| {
                                render_app(
                                    frame,
                                    &snapshot,
                                    &orchestrator_url,
                                    active_tab,
                                    portfolio_selected,
                                    &form,
                                )
                            })?;
                        }
                        EventOutcome::Continue => {}
                    }
                }
                Event::Paste(text) => {
                    if matches!(active_tab, ActiveTab::Deploy) {
                        if form.focus == 1 {
                            form.set_network_query(text.trim(), &snapshot.networks);
                        } else {
                            form.insert_text(&text);
                            if form.focus == 0 {
                                orchestrator_url = form.orchestrator_url.trim_end_matches('/').to_string();
                            }
                            if form.focus == 3 && form.context_path.trim().is_empty() {
                                let candidate = PathBuf::from(&form.dockerfile_path);
                                if let Some(parent) = candidate.parent() {
                                    form.context_path = parent.to_string_lossy().to_string();
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Check for submission status updates (non-blocking)
        loop {
            match submission_rx.try_recv() {
                Ok(msg) => {
                    match msg {
                        SubmissionMessage::StatusUpdate(status) => {
                            form.status_message = status;
                        }
                        SubmissionMessage::Success(result) => {
                            form.status_message = if let Some(url) = &result.deploy_url {
                                format!("✓ Deployed: {}", url)
                            } else {
                                "✓ Job submitted; public URL is still pending.".to_string()
                            };
                            form.last_result = Some(result);
                        }
                        SubmissionMessage::Error(err) => {
                            form.status_message = format!("✗ Deploy failed: {}", err);
                            form.last_result = Some(DeployResult {
                                job_id: "-".to_string(),
                                network_name: form
                                    .network(&snapshot.networks)
                                    .map(|network| network.name.clone())
                                    .unwrap_or_else(|| "-".to_string()),
                                network_id: form
                                    .network(&snapshot.networks)
                                    .map(|network| network.network_id.clone())
                                    .unwrap_or_else(|| "-".to_string()),
                                status: JobStatus::Failed,
                                assigned_node_id: None,
                                deploy_url: None,
                                message: err.clone(),
                                error_detail: Some(err),
                            });
                        }
                        SubmissionMessage::StopSuccess(msg) => {
                            form.status_message = format!("✓ Job stopped: {}", msg);
                            snapshot = refresh_snapshot(&client, &orchestrator_url).await;
                        }
                        SubmissionMessage::StopError(err) => {
                            form.status_message = format!("✗ Stop failed: {}", err);
                        }
                    }
                }
                Err(_) => {
                    // No messages available, that's fine
                    break;
                }
            }
        }

        // Check for background refresh updates (non-blocking)
        if let Ok(new_snapshot) = refresh_rx.try_recv() {
            snapshot = new_snapshot;
        }

        // Small sleep to avoid busy-waiting and keep responsive
        sleep(Duration::from_millis(16)).await;
    };

    let restore_result = restore_terminal(terminal);

    match (run_result, restore_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(err), _) => Err(err),
        (Ok(()), Err(err)) => Err(err.into()),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base_url = env::var("ORCHESTRATOR_URL")
        .or_else(|_| env::var("ORCHESTRATOR_BASE_URL"))
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());

    run(base_url).await
}
