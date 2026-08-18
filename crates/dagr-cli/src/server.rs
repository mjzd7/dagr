//! Embedded Zero-Dependency Web Dashboard and SSE Streaming Server

use dagr_core::{DagrError, Result, TelemetryStore, TimeWindow};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

pub static DASHBOARD_HTML: &str = include_str!("web/dashboard.html");

pub struct DashboardServer {
    pub workspace_root: PathBuf,
    pub port: u16,
    pub tx: broadcast::Sender<String>,
}

impl DashboardServer {
    pub async fn bind_and_run(
        workspace_root: PathBuf,
        preferred_port: Option<u16>,
        auto_open: bool,
    ) -> Result<()> {
        let (tx, _rx) = broadcast::channel(128);

        let start_port = preferred_port.unwrap_or(3333);
        let mut actual_port = start_port;
        let mut listener_opt = None;

        for p in start_port..(start_port + 10) {
            match TcpListener::bind(format!("127.0.0.1:{}", p)).await {
                Ok(l) => {
                    actual_port = p;
                    listener_opt = Some(l);
                    break;
                }
                Err(_) => continue,
            }
        }

        let listener = listener_opt.ok_or_else(|| {
            DagrError::Io(std::io::Error::new(
                std::io::ErrorKind::AddrInUse,
                format!("Ports {}..{} are all in use", start_port, start_port + 10),
            ))
        })?;

        let url = format!("http://127.0.0.1:{}", actual_port);
        eprintln!(
            "\n{}",
            colored::Colorize::bold(colored::Colorize::cyan(
                "⚡ DAGR Lifetime Telemetry & ROI Dashboard"
            ))
        );
        eprintln!(
            "   Local URL:    {}",
            colored::Colorize::bold(colored::Colorize::green(url.as_str()))
        );
        eprintln!("   Workspace:    {:?}", workspace_root);
        eprintln!("   Streaming:    Server-Sent Events (SSE) active");
        eprintln!("   Press Ctrl+C to stop.\n");

        if auto_open {
            #[cfg(target_os = "macos")]
            let _ = std::process::Command::new("open").arg(&url).spawn();
            #[cfg(target_os = "linux")]
            let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
            #[cfg(target_os = "windows")]
            let _ = std::process::Command::new("cmd")
                .args(["/C", "start", &url])
                .spawn();
        }

        let workspace_arc = Arc::new(workspace_root);

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let ws = Arc::clone(&workspace_arc);
                    let tx_clone = tx.clone();
                    tokio::spawn(async move {
                        let _ = handle_http_connection(stream, ws, tx_clone).await;
                    });
                }
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                }
            }
        }
    }
}

async fn handle_http_connection(
    mut stream: TcpStream,
    workspace: Arc<PathBuf>,
    tx: broadcast::Sender<String>,
) -> Result<()> {
    let mut buffer = [0u8; 4096];
    let n = stream.read(&mut buffer).await.map_err(DagrError::Io)?;
    if n == 0 {
        return Ok(());
    }

    let request_str = String::from_utf8_lossy(&buffer[..n]);
    let first_line = request_str.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    if parts.len() < 2 {
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    if method != "GET" {
        let resp = "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n";
        stream
            .write_all(resp.as_bytes())
            .await
            .map_err(DagrError::Io)?;
        return Ok(());
    }

    if path == "/" || path == "/index.html" {
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            DASHBOARD_HTML.len(),
            DASHBOARD_HTML
        );
        stream
            .write_all(resp.as_bytes())
            .await
            .map_err(DagrError::Io)?;
        return Ok(());
    }

    if path == "/api/stats" {
        let store = TelemetryStore::open(&workspace)?;
        let summary = store.get_summary(TimeWindow::Lifetime)?;
        let clients = store.get_client_breakdown()?;
        let velocity = store.get_daily_velocity(30)?;

        let payload = json!({
            "workspace": workspace.display().to_string(),
            "summary": summary,
            "clients": clients,
            "velocity": velocity
        });
        let body = serde_json::to_string(&payload)?;

        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(resp.as_bytes())
            .await
            .map_err(DagrError::Io)?;
        return Ok(());
    }

    if path == "/api/events" {
        let store = TelemetryStore::open(&workspace)?;
        let events = store.get_recent_events(50)?;
        let body = serde_json::to_string(&events)?;

        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(resp.as_bytes())
            .await
            .map_err(DagrError::Io)?;
        return Ok(());
    }

    if path == "/api/graph" {
        let db_path = workspace.join(".dagr").join("index.db");
        let nodes = if db_path.exists() {
            if let Ok(store) = dagr_core::LocalIndexStore::open(&workspace) {
                if let Ok(matches) = store.search_symbols("", 60) {
                    matches
                        .into_iter()
                        .enumerate()
                        .map(|(idx, m)| {
                            let angle = (idx as f64 * 0.4) % (std::f64::consts::PI * 2.0);
                            let radius = 120.0 + (idx as f64 * 4.0);
                            json!({
                                "id": m.id,
                                "name": m.symbol_name,
                                "type": format!("{:?}", m.kind).to_lowercase(),
                                "x": 400.0 + angle.cos() * radius,
                                "y": 260.0 + angle.sin() * radius,
                                "file": m.span.file_path
                            })
                        })
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let body = serde_json::to_string(&json!({ "nodes": nodes }))?;
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(resp.as_bytes())
            .await
            .map_err(DagrError::Io)?;
        return Ok(());
    }

    if path.starts_with("/api/export") {
        let is_csv = path.contains("format=csv");
        let store = TelemetryStore::open(&workspace)?;

        if is_csv {
            let csv = store.export_csv()?;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/csv\r\nContent-Disposition: attachment; filename=\"dagr_telemetry.csv\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                csv.len(),
                csv
            );
            stream
                .write_all(resp.as_bytes())
                .await
                .map_err(DagrError::Io)?;
        } else {
            let json_data = store.export_json()?;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Disposition: attachment; filename=\"dagr_telemetry.json\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                json_data.len(),
                json_data
            );
            stream
                .write_all(resp.as_bytes())
                .await
                .map_err(DagrError::Io)?;
        }
        return Ok(());
    }

    if path == "/api/stream" {
        let headers = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\nAccess-Control-Allow-Origin: *\r\n\r\n";
        stream
            .write_all(headers.as_bytes())
            .await
            .map_err(DagrError::Io)?;

        let mut rx = tx.subscribe();
        let initial_ping = "data: {\"type\": \"connected\"}\n\n";
        stream
            .write_all(initial_ping.as_bytes())
            .await
            .map_err(DagrError::Io)?;

        loop {
            tokio::select! {
                msg_res = rx.recv() => {
                    match msg_res {
                        Ok(msg) => {
                            let formatted = format!("data: {}\n\n", msg);
                            if stream.write_all(formatted.as_bytes()).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(15)) => {
                    // Send periodic SSE heartbeat ping
                    if stream.write_all(b": ping\n\n").await.is_err() {
                        break;
                    }
                }
            }
        }

        return Ok(());
    }

    let not_found = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
    stream
        .write_all(not_found.as_bytes())
        .await
        .map_err(DagrError::Io)?;
    Ok(())
}
