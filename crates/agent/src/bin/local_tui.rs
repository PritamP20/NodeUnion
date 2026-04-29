use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nodeunion_agent::models::{AgentStateResponse, NodeStatus, RunningChunkView};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use reqwest::Client;
use std::env;
use std::io::{self, Stdout};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
struct ProviderSettlementView {
    provider_settlement_id: String,
    job_id: String,
    provider_wallet: String,
    network_id: String,
    units_earned: i64,
    amount_tokens: i64,
    tx_hash: Option<String>,
    tx_status: Option<String>,
    created_at_epoch_secs: i64,
}

#[derive(Clone, Debug)]
struct PortfolioSnapshot {
    provider_wallet: String,
    settlement_count: usize,
    total_amount_tokens: i64,
    recent_settlements: Vec<ProviderSettlementView>,
}

struct Snapshot {
    healthy: bool,
    state: Option<AgentStateResponse>,
    provider_wallet: Option<String>,
    portfolio: Option<PortfolioSnapshot>,
    errors: Vec<String>,
    fetched_at_epoch: u64,
}

async fn fetch_snapshot(
    client: &Client,
    base_url: &str,
    orchestrator_url: Option<&str>,
    provider_wallet: Option<&str>,
) -> Snapshot {
    let mut errors = Vec::new();

    let health_url = format!("{}/health", base_url);
    let healthy = match client.get(&health_url).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(err) => {
            errors.push(format!("health check failed: {}", err));
            false
        }
    };

    let state = match fetch_json::<AgentStateResponse>(client, base_url, "/state").await {
        Ok(v) => Some(v),
        Err(err) => {
            errors.push(err);
            None
        }
    };

    let portfolio = match (orchestrator_url, provider_wallet) {
        (Some(orchestrator_base_url), Some(wallet)) if !wallet.trim().is_empty() => {
            match fetch_provider_portfolio(client, orchestrator_base_url, wallet).await {
                Ok(portfolio) => Some(portfolio),
                Err(err) => {
                    errors.push(err);
                    None
                }
            }
        }
        _ => None,
    };

    let fetched_at_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    Snapshot {
        healthy,
        state,
        provider_wallet: provider_wallet.map(|value| value.to_string()),
        portfolio,
        errors,
        fetched_at_epoch,
    }
}

async fn fetch_provider_portfolio(
    client: &Client,
    orchestrator_base_url: &str,
    provider_wallet: &str,
) -> Result<PortfolioSnapshot, String> {
    let url = format!(
        "{}/providers/{}/settlements",
        orchestrator_base_url.trim_end_matches('/'),
        provider_wallet
    );

    let settlements = client
        .get(&url)
        .send()
        .await
        .map_err(|err| format!("GET /providers/{}/settlements failed: {}", provider_wallet, err))?
        .error_for_status()
        .map_err(|err| format!("provider settlements request failed: {}", err))?
        .json::<Vec<ProviderSettlementView>>()
        .await
        .map_err(|err| format!("parse provider settlements JSON failed: {}", err))?;

    let total_amount_tokens = settlements.iter().map(|settlement| settlement.amount_tokens).sum();

    Ok(PortfolioSnapshot {
        provider_wallet: provider_wallet.to_string(),
        settlement_count: settlements.len(),
        total_amount_tokens,
        recent_settlements: settlements.into_iter().take(4).collect(),
    })
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

fn percent_ratio(value: f64) -> f64 {
    (value / 100.0).clamp(0.0, 1.0)
}

fn node_status_style(status: &NodeStatus) -> Style {
    match status {
        NodeStatus::Idle => Style::default().fg(Color::Green),
        NodeStatus::Busy => Style::default().fg(Color::Yellow),
        NodeStatus::Draining => Style::default().fg(Color::Magenta),
        NodeStatus::Preempting => Style::default().fg(Color::Red),
    }
}

fn node_status_label(status: &NodeStatus) -> &'static str {
    match status {
        NodeStatus::Idle => "IDLE",
        NodeStatus::Busy => "BUSY",
        NodeStatus::Draining => "DRAINING",
        NodeStatus::Preempting => "PREEMPTING",
    }
}

fn format_tokens(value: i64) -> String {
    let negative = value.is_negative();
    let digits = value.abs().to_string();
    let mut grouped = String::new();

    for (index, ch) in digits.chars().rev().enumerate() {
        if index != 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }

    let mut formatted = grouped.chars().rev().collect::<String>();
    if negative {
        formatted.insert(0, '-');
    }

    format!("{} tokens", formatted)
}

fn short_wallet(wallet: &str) -> String {
    if wallet.len() <= 12 {
        wallet.to_string()
    } else {
        format!("{}…{}", &wallet[..6], &wallet[wallet.len().saturating_sub(4)..])
    }
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

fn render_portfolio_card(area: Rect, frame: &mut Frame<'_>, portfolio: Option<&PortfolioSnapshot>, provider_wallet: Option<&str>) {
    let mut lines = Vec::new();

    match provider_wallet {
        Some(wallet) => {
            lines.push(Line::from(vec![
                Span::styled("wallet ", Style::default().fg(Color::DarkGray)),
                Span::styled(wallet.to_string(), Style::default().add_modifier(Modifier::BOLD)),
            ]));
        }
        None => {
            lines.push(Line::from(vec![
                Span::styled("wallet ", Style::default().fg(Color::DarkGray)),
                Span::styled("(not set)", Style::default().fg(Color::Yellow)),
            ]));
        }
    }

    match portfolio {
        Some(portfolio) => {
            lines.push(Line::from(vec![
                Span::styled("earned ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format_tokens(portfolio.total_amount_tokens),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("settlements ", Style::default().fg(Color::DarkGray)),
                Span::raw(portfolio.settlement_count.to_string()),
            ]));

            if portfolio.recent_settlements.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("recent ", Style::default().fg(Color::DarkGray)),
                    Span::styled("none yet", Style::default().fg(Color::Yellow)),
                ]));
            } else {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("recent payouts", Style::default().fg(Color::DarkGray)),
                ]));

                for settlement in &portfolio.recent_settlements {
                    let status = settlement
                        .tx_status
                        .clone()
                        .unwrap_or_else(|| "pending".to_string());
                    let tx = settlement
                        .tx_hash
                        .clone()
                        .map(|value| short_wallet(&value))
                        .unwrap_or_else(|| "no tx".to_string());

                    lines.push(Line::from(vec![
                        Span::styled("• ", Style::default().fg(Color::DarkGray)),
                        Span::raw(short_text(&settlement.job_id, 12)),
                        Span::styled("  ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format_tokens(settlement.amount_tokens), Style::default().fg(Color::Cyan)),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::default().fg(Color::DarkGray)),
                        Span::styled(status, Style::default().fg(Color::Yellow)),
                        Span::styled("  ", Style::default().fg(Color::DarkGray)),
                        Span::styled(tx, Style::default().fg(Color::DarkGray)),
                    ]));
                }
            }
        }
        None => {
            lines.push(Line::from(vec![
                Span::styled("earned ", Style::default().fg(Color::DarkGray)),
                Span::styled("--", Style::default().fg(Color::Yellow)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("status ", Style::default().fg(Color::DarkGray)),
                Span::raw("set PROVIDER_WALLET and ORCHESTRATOR_URL to load portfolio"),
            ]));
        }
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("Portfolio"))
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn compact_chunk(chunk: &RunningChunkView) -> Vec<Line<'static>> {
    let mut container_id = chunk.container_id.clone();
    if container_id.len() > 16 {
        container_id.truncate(16);
        container_id.push_str("...");
    }

    vec![
        Line::from(vec![
            Span::styled("job ", Style::default().fg(Color::DarkGray)),
            Span::raw(chunk.job_id.clone()),
            Span::styled("  chunk ", Style::default().fg(Color::DarkGray)),
            Span::raw(chunk.chunk_id.clone()),
        ]),
        Line::from(vec![
            Span::styled("status ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:?}", chunk.status), Style::default().fg(Color::Cyan)),
            Span::styled("  container ", Style::default().fg(Color::DarkGray)),
            Span::raw(container_id),
        ]),
    ]
}

fn render_header(area: Rect, frame: &mut Frame<'_>, snapshot: &Snapshot, base_url: &str) {
    let health_color = if snapshot.healthy { Color::Green } else { Color::Red };
    let status_text = if snapshot.healthy { "ONLINE" } else { "OFFLINE" };

    let title = Line::from(vec![
        Span::styled("NodeUnion Agent Dashboard ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("● ", Style::default().fg(health_color).add_modifier(Modifier::BOLD)),
        Span::styled(status_text, Style::default().fg(health_color).add_modifier(Modifier::BOLD)),
    ]);

    let mut body = vec![
        Line::from(vec![
            Span::styled("agent ", Style::default().fg(Color::DarkGray)),
            Span::raw(base_url.to_string()),
            Span::styled("   refresh ", Style::default().fg(Color::DarkGray)),
            Span::raw(snapshot.fetched_at_epoch.to_string()),
        ]),
        Line::from(vec![
            Span::styled("wallet ", Style::default().fg(Color::DarkGray)),
            Span::raw(
                snapshot
                    .provider_wallet
                    .as_deref()
                    .map(short_wallet)
                    .unwrap_or_else(|| "(not set)".to_string()),
            ),
        ]),
        Line::from(vec![
            Span::styled("controls ", Style::default().fg(Color::DarkGray)),
            Span::raw("ctrl+c to exit"),
        ]),
    ];

    if !snapshot.errors.is_empty() {
        body.push(Line::from(vec![
            Span::styled("warnings ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", snapshot.errors.len()), Style::default().fg(Color::Yellow)),
            Span::raw(" request issue(s)"),
        ]));
    }

    let paragraph = Paragraph::new(Text::from(body))
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn render_metric_cards(area: Rect, frame: &mut Frame<'_>, state: &AgentStateResponse) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(area);

    let cpu_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("CPU"))
        .gauge_style(Style::default().fg(Color::Green))
        .ratio(percent_ratio(state.cpu_usage_pct as f64))
        .label(format!("{:.1}% used", state.cpu_usage_pct));

    let ram_used = state.ram_total_mb.saturating_sub(state.ram_available_mb);
    let ram_ratio = if state.ram_total_mb == 0 {
        0.0
    } else {
        ram_used as f64 / state.ram_total_mb as f64
    };
    let ram_label = if state.ram_total_mb == 0 {
        "n/a".to_string()
    } else {
        format!("{} / {} MB", ram_used, state.ram_total_mb)
    };

    let ram_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Memory"))
        .gauge_style(Style::default().fg(Color::Cyan))
        .ratio(ram_ratio.clamp(0.0, 1.0))
        .label(ram_label);

    let disk_card = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("free disk ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} GB", state.disk_available_gb),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("running chunks ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                state.running_chunks.to_string(),
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("avg cpu window ", Style::default().fg(Color::DarkGray)),
            Span::raw(
                state
                    .avg_cpu_window_pct
                    .map(|value| format!("{:.1}%", value))
                    .unwrap_or_else(|| "n/a".to_string()),
            ),
        ]),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Storage & Load"))
    .wrap(Wrap { trim: true });

    frame.render_widget(cpu_gauge, columns[0]);
    frame.render_widget(ram_gauge, columns[1]);
    frame.render_widget(disk_card, columns[2]);
}

fn render_status_area(
    area: Rect,
    frame: &mut Frame<'_>,
    state: &AgentStateResponse,
    portfolio: Option<&PortfolioSnapshot>,
    provider_wallet: Option<&str>,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(36), Constraint::Percentage(34), Constraint::Percentage(30)])
        .split(area);

    let summary = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("node ", Style::default().fg(Color::DarkGray)),
            Span::styled(state.node_id.clone(), Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("public url ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                state
                    .public_url
                    .clone()
                    .unwrap_or_else(|| "(not available yet)".to_string()),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("state ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                node_status_label(&state.status),
                node_status_style(&state.status).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("idle ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if state.is_idle { "yes" } else { "no" },
                Style::default().fg(if state.is_idle { Color::Green } else { Color::Yellow }),
            ),
        ]),
        Line::from(vec![
            Span::styled("preempt spikes ", Style::default().fg(Color::DarkGray)),
            Span::raw(state.consecutive_preempt_spikes.to_string()),
        ]),
        Line::from(vec![
            Span::styled("avg cpu window ", Style::default().fg(Color::DarkGray)),
            Span::raw(
                state
                    .avg_cpu_window_pct
                    .map(|value| format!("{:.1}%", value))
                    .unwrap_or_else(|| "n/a".to_string()),
            ),
        ]),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Node State"))
    .wrap(Wrap { trim: true });

    let chunks_list = if state.active_chunks.is_empty() {
        List::new(vec![ListItem::new("No jobs are running on this node.")])
    } else {
        List::new(
            state
                .active_chunks
                .iter()
                .take(8)
                .map(|chunk| ListItem::new(compact_chunk(chunk)))
                .collect::<Vec<_>>(),
        )
    }
    .block(Block::default().borders(Borders::ALL).title("Active Jobs"))
    .highlight_style(Style::default().fg(Color::Cyan));

    frame.render_widget(summary, chunks[0]);
    frame.render_widget(chunks_list, chunks[1]);
    render_portfolio_card(chunks[2], frame, portfolio, provider_wallet);
}

fn render_errors(area: Rect, frame: &mut Frame<'_>, errors: &[String]) {
    let block = Block::default().borders(Borders::ALL).title("Diagnostics");

    if errors.is_empty() {
        let paragraph = Paragraph::new("No errors reported.").block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let items = errors
        .iter()
        .map(|err| ListItem::new(Line::from(vec![Span::styled(err.clone(), Style::default().fg(Color::Red))])))
        .collect::<Vec<_>>();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_dashboard(frame: &mut Frame<'_>, snapshot: &Snapshot, base_url: &str) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(9),
            Constraint::Min(10),
        ])
        .split(frame.area());

    render_header(outer[0], frame, snapshot, base_url);

    if let Some(state) = &snapshot.state {
        render_metric_cards(outer[1], frame, state);
        let body = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(7), Constraint::Length(7)])
            .split(outer[2]);
        render_status_area(
            body[0],
            frame,
            state,
            snapshot.portfolio.as_ref(),
            snapshot.provider_wallet.as_deref(),
        );
        render_errors(body[1], frame, &snapshot.errors);
    } else {
        let body = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(7)])
            .split(outer[2]);

        let empty = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "Node state is unavailable.",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from("Check the agent health, Docker daemon, and orchestrator registration."),
        ]))
        .block(Block::default().borders(Borders::ALL).title("Node State"))
        .wrap(Wrap { trim: true });
        frame.render_widget(empty, body[0]);
        render_errors(body[1], frame, &snapshot.errors);
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

#[tokio::main]
async fn main() -> io::Result<()> {
    let base_url = env::var("AGENT_URL").unwrap_or_else(|_| "http://127.0.0.1:8090".to_string());
    let base_url = base_url.trim_end_matches('/').to_string();
    let orchestrator_url = env::var("ORCHESTRATOR_URL")
        .or_else(|_| env::var("ORCHESTRATOR_BASE_URL"))
        .ok();
    let provider_wallet = env::var("PROVIDER_WALLET").ok().and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });
    let client = Client::new();
    let mut terminal = setup_terminal()?;

    let run_result = loop {
        let snapshot = fetch_snapshot(
            &client,
            &base_url,
            orchestrator_url.as_deref(),
            provider_wallet.as_deref(),
        )
        .await;
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
