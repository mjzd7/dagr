//! Interactive Terminal User Interface (TUI) Dashboard for DAGR Telemetry

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dagr_core::{Result, TelemetryStore, TimeWindow};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Sparkline, Table},
    Frame, Terminal,
};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub struct TuiApp {
    pub workspace_root: PathBuf,
    pub should_quit: bool,
    pub last_refresh: Instant,
    pub open_web_requested: bool,
    pub export_requested: bool,
}

impl TuiApp {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            should_quit: false,
            last_refresh: Instant::now(),
            open_web_requested: false,
            export_requested: false,
        }
    }
}

pub fn run_tui(workspace_root: &Path) -> Result<()> {
    enable_raw_mode().map_err(dagr_core::DagrError::Io)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(dagr_core::DagrError::Io)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(dagr_core::DagrError::Io)?;

    let mut app = TuiApp::new(workspace_root.to_path_buf());
    let res = run_loop(&mut terminal, &mut app);

    // Restore terminal cleanly
    disable_raw_mode().map_err(dagr_core::DagrError::Io)?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .map_err(dagr_core::DagrError::Io)?;
    terminal.show_cursor().map_err(dagr_core::DagrError::Io)?;

    if app.open_web_requested {
        eprintln!("Launching Web Dashboard...");
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open")
            .arg("http://127.0.0.1:3333")
            .spawn();
    }

    if app.export_requested {
        if let Ok(store) = TelemetryStore::open(workspace_root) {
            if let Ok(json) = store.export_json() {
                let out_path = workspace_root.join("dagr_telemetry.json");
                let _ = std::fs::write(&out_path, json);
                eprintln!("Exported telemetry to {:?}", out_path);
            }
        }
    }

    res
}

fn run_loop<B: Backend>(terminal: &mut Terminal<B>, app: &mut TuiApp) -> Result<()> {
    loop {
        terminal
            .draw(|f| ui(f, app))
            .map_err(dagr_core::DagrError::Io)?;

        if event::poll(Duration::from_millis(200)).map_err(dagr_core::DagrError::Io)? {
            if let Event::Key(key) = event::read().map_err(dagr_core::DagrError::Io)? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.should_quit = true;
                    }
                    KeyCode::Char('w') => {
                        app.open_web_requested = true;
                        app.should_quit = true;
                    }
                    KeyCode::Char('e') => {
                        app.export_requested = true;
                        app.should_quit = true;
                    }
                    KeyCode::Char('r') => {
                        app.last_refresh = Instant::now();
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &TuiApp) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(6),  // KPI Stats
            Constraint::Length(10), // Velocity Chart & Clients
            Constraint::Min(8),     // Recent Events
            Constraint::Length(3),  // Footer Keybindings
        ])
        .split(size);

    // 1. Header
    let header_text = vec![Line::from(vec![
        Span::styled(
            "⚡ DAGR HYPERVISOR ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("v0.1.0  ", Style::default().fg(Color::Yellow)),
        Span::styled("• Workspace: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            app.workspace_root.display().to_string(),
            Style::default().fg(Color::White),
        ),
    ])];
    let header = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(header, chunks[0]);

    // Query Store
    let store_res = TelemetryStore::open(&app.workspace_root);
    let (summary, clients, recent, velocity) = if let Ok(ref store) = store_res {
        (
            store.get_summary(TimeWindow::Lifetime).ok(),
            store.get_client_breakdown().ok(),
            store.get_recent_events(8).ok(),
            store.get_daily_velocity(14).ok(),
        )
    } else {
        (None, None, None, None)
    };

    // 2. KPI Metrics Grid
    let kpi_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[1]);

    let tokens_saved_str = summary
        .as_ref()
        .map(|s| s.total_tokens_saved.to_string())
        .unwrap_or_else(|| "0".into());
    let usd_saved_str = summary
        .as_ref()
        .map(|s| format!("${:.2} USD", s.estimated_usd_saved))
        .unwrap_or_else(|| "$0.00".into());
    let comp_ratio = summary
        .as_ref()
        .map(|s| s.overall_compression_ratio)
        .unwrap_or(0.0);
    let slices_count_str = summary
        .as_ref()
        .map(|s| s.total_slices.to_string())
        .unwrap_or_else(|| "0".into());

    let card1 = Paragraph::new(vec![
        Line::from(Span::styled(
            "LIFETIME TOKENS SAVED",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            tokens_saved_str,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(card1, kpi_chunks[0]);

    let card2 = Paragraph::new(vec![
        Line::from(Span::styled(
            "ESTIMATED ROI SAVINGS",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            usd_saved_str,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(card2, kpi_chunks[1]);

    let card3 = Gauge::default()
        .block(
            Block::default()
                .title("AVG COMPRESSION")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .percent((comp_ratio * 100.0).min(100.0) as u16)
        .label(format!("{:.1}%", comp_ratio * 100.0));
    f.render_widget(card3, kpi_chunks[2]);

    let card4 = Paragraph::new(vec![
        Line::from(Span::styled(
            "SLICES SERVED (<0.3ms)",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            slices_count_str,
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(card4, kpi_chunks[3]);

    // 3. Middle Section: Velocity Sparkline + Client Breakdown
    let mid_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[2]);

    let sparkline_data: Vec<u64> = velocity
        .as_ref()
        .map(|v| v.iter().map(|p| p.tokens_saved as u64).collect())
        .unwrap_or_else(|| vec![0]);
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(" 14-DAY SAVINGS VELOCITY ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::Green))
        .data(&sparkline_data);
    f.render_widget(sparkline, mid_chunks[0]);

    // Client Breakdown Table
    let client_rows: Vec<Row> = clients
        .as_ref()
        .map(|list| {
            list.iter()
                .map(|c| {
                    Row::new(vec![
                        Cell::from(c.client_id.clone()).style(Style::default().fg(Color::Cyan)),
                        Cell::from(c.tokens_saved.to_string())
                            .style(Style::default().fg(Color::Green)),
                        Cell::from(format!("{:.1}%", c.percentage)),
                    ])
                })
                .collect()
        })
        .unwrap_or_default();

    let client_table = Table::new(
        client_rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Constraint::Percentage(20),
        ],
    )
    .header(Row::new(vec!["Client", "Saved", "%"]).style(Style::default().fg(Color::DarkGray)))
    .block(
        Block::default()
            .title(" TOP AI CLIENTS ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(client_table, mid_chunks[1]);

    // 4. Bottom Section: Recent Slicing Events Table
    let event_rows: Vec<Row> = recent
        .as_ref()
        .map(|list| {
            list.iter()
                .map(|e| {
                    Row::new(vec![
                        Cell::from(e.client_id.clone()).style(Style::default().fg(Color::Cyan)),
                        Cell::from(e.symbol_name.clone().unwrap_or_else(|| "-".into()))
                            .style(Style::default().fg(Color::Yellow)),
                        Cell::from(e.raw_tokens.to_string()),
                        Cell::from(e.sliced_tokens.to_string()),
                        Cell::from(format!("+{}", e.tokens_saved)).style(
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Cell::from(format!("{:.2}ms", e.latency_us as f64 / 1000.0))
                            .style(Style::default().fg(Color::Magenta)),
                    ])
                })
                .collect()
        })
        .unwrap_or_default();

    let events_table = Table::new(
        event_rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(35),
            Constraint::Percentage(12),
            Constraint::Percentage(12),
            Constraint::Percentage(14),
            Constraint::Percentage(12),
        ],
    )
    .header(
        Row::new(vec![
            "Client", "Symbol", "Raw", "Sliced", "Saved", "Latency",
        ])
        .style(Style::default().fg(Color::DarkGray)),
    )
    .block(
        Block::default()
            .title(" LIVE RECENT AST SLICES ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(events_table, chunks[3]);

    // 5. Footer Keybindings
    let footer_text = vec![Line::from(vec![
        Span::styled(
            "[Q] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Quit  "),
        Span::styled(
            "[W] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Launch Web  "),
        Span::styled(
            "[R] ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Refresh  "),
        Span::styled(
            "[E] ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Export JSON  "),
    ])];
    let footer = Paragraph::new(footer_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(footer, chunks[4]);
}
