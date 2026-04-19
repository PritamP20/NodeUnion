use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crate::model::{JobRecord, JobStatus, NetworkRecord, NodeRecord, NodeStatus};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Row, Table, Wrap},
    Terminal,
};
use reqwest::Client;
use std::io::{self, Stdout};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

struct Snapshot {
    healthy: bool,
    networks: Vec<NetworkRecord>,
    nodes: Vec<NodeRecord>,
    jobs: Vec<JobRecord>,
    errors: Vec<String>,
    fetched_at_epoch: u64,
}

async fn fetch_snapshot(client: &Client, base_url: &str) -> Snapshot {
    let mut errors = Vec::new();

    let health_url = format!("{}/health", base_url);
    let healthy = match client.get(&health_url).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(err) => {
            errors.push(format!("health check failed: {}", err));
            false
        }
    };

    let networks = match fetch_json::<Vec<NetworkRecord>>(client, base_url, "/networks").await {
        Ok(v) => v,
        Err(err) => {
            errors.push(err);
            Vec::new()
        }
    };

    let nodes = match fetch_json::<Vec<NodeRecord>>(client, base_url, "/nodes").await {
        Ok(v) => v,
        Err(err) => {
            errors.push(err);
            Vec::new()
        }
    };

    let jobs = match fetch_json::<Vec<JobRecord>>(client, base_url, "/jobs").await {
        Ok(v) => v,
        Err(err) => {
            errors.push(err);
            Vec::new()
        }
    };

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

async fn fetch_json<T>(client: &Client, base_url: &str, path: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let url = format!("{}{}", base_url, path);
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|err| format!("GET {} failed: {}", path, err))?;

    if !resp.status().is_success() {
        return Err(format!("GET {} returned status {}", path, resp.status()));
    }

    resp.json::<T>()
        .await
        .map_err(|err| format!("parse {} JSON failed: {}", path, err))
}

fn node_status_style(status: &NodeStatus) -> Style {
    match status {
        NodeStatus::Idle => Style::default().fg(Color::Green),
        NodeStatus::Busy => Style::default().fg(Color::Yellow),
        NodeStatus::Draining => Style::default().fg(Color::Magenta),
        NodeStatus::Preempting => Style::default().fg(Color::Red),
        NodeStatus::Offline => Style::default().fg(Color::DarkGray),
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

fn percent_ratio(value: f64) -> f64 {
    (value / 100.0).clamp(0.0, 1.0)
}

fn node_status_label(status: &NodeStatus) -> &'static str {
    match status {
        NodeStatus::Idle => "IDLE",
        NodeStatus::Busy => "BUSY",
        NodeStatus::Draining => "DRAINING",
        NodeStatus::Preempting => "PREEMPTING",
        NodeStatus::Offline => "OFFLINE",
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

fn node_age_secs(node: &NodeRecord, now: u64) -> u64 {
    now.saturating_sub(node.last_seen_epoch_secs)
}

fn compact_agent_url(url: &str, max_len: usize) -> String {
    if url.len() <= max_len {
        return url.to_string();
    }

    if max_len <= 3 {
        return "...".to_string();
    }

    format!("{}...", &url[..max_len - 3])
}

fn snapshot_public_url(snapshot: &Snapshot) -> Option<String> {
    snapshot
        .networks
        .iter()
        .find_map(|network| network.orchestrator_url.clone())
        .map(|url| compact_agent_url(&url, 72))
}

fn render_header(area: Rect, frame: &mut Frame<'_>, snapshot: &Snapshot, base_url: &str) {
    let health_color = if snapshot.healthy { Color::Green } else { Color::Red };
    let status_text = if snapshot.healthy { "ONLINE" } else { "OFFLINE" };

    let title = Line::from(vec![
        Span::styled("NodeUnion Cloud Dashboard ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("● ", Style::default().fg(health_color).add_modifier(Modifier::BOLD)),
        Span::styled(status_text, Style::default().fg(health_color).add_modifier(Modifier::BOLD)),
    ]);

    let public_url = snapshot_public_url(snapshot).unwrap_or_else(|| "(not available yet)".to_string());

    let body = vec![
        Line::from(vec![
            Span::styled("orchestrator ", Style::default().fg(Color::DarkGray)),
            Span::raw(base_url.to_string()),
            Span::styled("   refresh ", Style::default().fg(Color::DarkGray)),
            Span::raw(snapshot.fetched_at_epoch.to_string()),
        ]),
        Line::from(vec![
            Span::styled("public url ", Style::default().fg(Color::DarkGray)),
            Span::styled(public_url, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("controls ", Style::default().fg(Color::DarkGray)),
            Span::raw("ctrl+c or q to exit"),
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
    let running_chunks: usize = snapshot.nodes.iter().map(|node| node.running_chunks).sum();

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let cloud_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Cloud Load"))
        .gauge_style(Style::default().fg(Color::Green))
        .ratio(if total_nodes == 0 { 0.0 } else { busy_nodes as f64 / total_nodes as f64 })
        .label(format!("{} busy / {} total", busy_nodes, total_nodes));

    let jobs_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Job Activity"))
        .gauge_style(Style::default().fg(Color::Cyan))
        .ratio(if snapshot.jobs.is_empty() { 0.0 } else { running_jobs as f64 / snapshot.jobs.len() as f64 })
        .label(format!("{} active / {} total", running_jobs, snapshot.jobs.len()));

    let cpu_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Avg CPU Available"))
        .gauge_style(Style::default().fg(Color::Yellow))
        .ratio(percent_ratio(avg_cpu_available))
        .label(format!("{:.1}% free", avg_cpu_available));

    let chunks_card = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("idle nodes ", Style::default().fg(Color::DarkGray)),
            Span::styled(idle_nodes.to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("running chunks ", Style::default().fg(Color::DarkGray)),
            Span::styled(running_chunks.to_string(), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("networks ", Style::default().fg(Color::DarkGray)),
            Span::styled(snapshot.networks.len().to_string(), Style::default().add_modifier(Modifier::BOLD)),
        ]),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Summary"))
    .wrap(Wrap { trim: true });

    frame.render_widget(cloud_gauge, columns[0]);
    frame.render_widget(jobs_gauge, columns[1]);
    frame.render_widget(cpu_gauge, columns[2]);
    frame.render_widget(chunks_card, columns[3]);
}

fn render_nodes(area: Rect, frame: &mut Frame<'_>, nodes: &[NodeRecord]) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    let rows = if nodes.is_empty() {
        vec![Row::new(vec!["No nodes registered yet", "-", "-", "-", "-", "-", "-", "-"])]
    } else {
        nodes
            .iter()
            .take(8)
            .map(|node| {
                Row::new(vec![
                    node.node_id.clone(),
                    node.network_id.clone(),
                    node_status_label(&node.status).to_string(),
                    if node.is_idle { "yes" } else { "no" }.to_string(),
                    format!("{:.1}%", node.cpu_available_pct),
                    format!("{} MB", node.ram_available_mb),
                    format!("{}", node.running_chunks),
                    compact_agent_url(&node.agent_url, 44),
                ])
                .style(node_status_style(&node.status))
            })
            .collect()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(18),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Length(11),
            Constraint::Length(14),
            Constraint::Length(8),
            Constraint::Min(28),
        ],
    )
    .header(
        Row::new(vec![
            "Node",
            "Network",
            "Status",
            "Idle",
            "CPU Avail",
            "RAM Avail",
            "Chunks",
            "Public URL",
        ])
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).title("Nodes"))
    .column_spacing(1);

    frame.render_widget(table, area);

    if !nodes.is_empty() {
        let footer = Paragraph::new(Text::from(vec![Line::from(vec![
            Span::styled("last seen age ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!("{}s max / {}s min", node_age_secs(&nodes[0], now), node_age_secs(nodes.last().unwrap(), now))),
        ])]))
        .block(Block::default().borders(Borders::ALL).title("Node Timing"))
        .wrap(Wrap { trim: true });

        let footer_area = Rect {
            x: area.x,
            y: area.y.saturating_add(area.height.saturating_sub(3)),
            width: area.width,
            height: 3,
        };

        if area.height > 4 {
            frame.render_widget(footer, footer_area);
        }
    }
}

fn render_jobs(area: Rect, frame: &mut Frame<'_>, jobs: &[JobRecord]) {
    let rows = if jobs.is_empty() {
        vec![Row::new(vec!["No jobs found", "-", "-", "-", "-"])]
    } else {
        jobs
            .iter()
            .take(8)
            .map(|job| {
                Row::new(vec![
                    job.job_id.clone(),
                    job_status_label(&job.status).to_string(),
                    job.assigned_node_id.clone().unwrap_or_else(|| "-".to_string()),
                    format!("{:.1} CPU", job.cpu_limit),
                    format!("{} MB", job.ram_limit_mb),
                ])
                .style(job_status_style(&job.status))
            })
            .collect()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Length(18),
            Constraint::Length(12),
            Constraint::Length(12),
        ],
    )
    .header(
        Row::new(vec!["Job", "Status", "Node", "CPU", "RAM"])
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).title("Jobs"))
    .column_spacing(1);

    frame.render_widget(table, area);
}

fn render_errors(area: Rect, frame: &mut Frame<'_>, errors: &[String]) {
    let block = Block::default().borders(Borders::ALL).title("Diagnostics");

    if errors.is_empty() {
        frame.render_widget(Paragraph::new("No errors reported.").block(block), area);
        return;
    }

    let items = errors
        .iter()
        .map(|error| ListItem::new(Line::from(vec![Span::styled(error.clone(), Style::default().fg(Color::Red))])))
        .collect::<Vec<_>>();

    frame.render_widget(List::new(items).block(block), area);
}

fn render_dashboard(frame: &mut Frame<'_>, snapshot: &Snapshot, base_url: &str) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Min(10),
        ])
        .split(frame.area());

    render_header(outer[0], frame, snapshot, base_url);
    render_summary(outer[1], frame, snapshot);

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(7)])
        .split(outer[2]);

    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(body[0]);

    render_nodes(split[0], frame, &snapshot.nodes);
    render_jobs(split[1], frame, &snapshot.jobs);
    render_errors(body[1], frame, &snapshot.errors);
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

pub async fn run(base_url: String) -> io::Result<()> {
    let base_url = base_url.trim_end_matches('/').to_string();
    let client = Client::new();
    let mut terminal = setup_terminal()?;

    let run_result = loop {
        let snapshot = fetch_snapshot(&client, &base_url).await;
        terminal.draw(|frame| render_dashboard(frame, &snapshot, &base_url))?;

        if should_exit()? {
            break Ok(());
        }

        tokio::select! {
            _ = sleep(Duration::from_secs(2)) => {}
            _ = tokio::signal::ctrl_c() => {
                break Ok(());
            }
        }
    };

    let restore_result = restore_terminal(terminal);
    run_result.and(restore_result)
}