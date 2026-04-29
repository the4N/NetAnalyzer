// ─────────────────────────────────────────────────────────────
// NetAnalyzer - Port Scanner Engine
// ─────────────────────────────────────────────────────────────

use std::net::Ipv4Addr;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};

use super::services::get_service_name;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortScanResult {
    pub port: u16,
    pub state: PortState,
    pub service: String,
    pub banner: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortState {
    Open,
    Closed,
    Filtered,
}

impl std::fmt::Display for PortState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortState::Open => write!(f, "Open"),
            PortState::Closed => write!(f, "Closed"),
            PortState::Filtered => write!(f, "Filtered"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PortScanProgress {
    Update { scanned: usize, total: usize },
    Found(PortScanResult),
    Done,
    Error(String),
}

/// Scan ports on a target host
pub async fn scan_ports(
    target: Ipv4Addr,
    ports: Vec<u16>,
    timeout: Duration,
    max_workers: usize,
    grab_banner: bool,
    tx: mpsc::UnboundedSender<PortScanProgress>,
) {
    let total = ports.len();
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_workers));
    let scanned = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let mut handles = Vec::new();

    for port in ports {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let tx = tx.clone();
        let scanned = scanned.clone();

        let handle = tokio::spawn(async move {
            let result = scan_single_port(target, port, timeout, grab_banner).await;
            let count = scanned.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;

            let _ = tx.send(PortScanProgress::Update {
                scanned: count,
                total,
            });

            if result.state == PortState::Open {
                let _ = tx.send(PortScanProgress::Found(result));
            }

            drop(permit);
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let _ = tx.send(PortScanProgress::Done);
}

async fn scan_single_port(
    target: Ipv4Addr,
    port: u16,
    timeout: Duration,
    grab_banner: bool,
) -> PortScanResult {
    let addr = format!("{}:{}", target, port);
    let service = get_service_name(port).to_string();

    match tokio::time::timeout(timeout, TcpStream::connect(&addr)).await {
        Ok(Ok(mut stream)) => {
            let banner = if grab_banner {
                grab_service_banner(&mut stream, port).await
            } else {
                String::new()
            };

            PortScanResult {
                port,
                state: PortState::Open,
                service,
                banner,
            }
        }
        Ok(Err(e)) => {
            let state = if e.kind() == std::io::ErrorKind::ConnectionRefused {
                PortState::Closed
            } else {
                PortState::Filtered
            };
            PortScanResult {
                port,
                state,
                service,
                banner: String::new(),
            }
        }
        Err(_) => {
            // Timeout = likely filtered
            PortScanResult {
                port,
                state: PortState::Filtered,
                service,
                banner: String::new(),
            }
        }
    }
}

async fn grab_service_banner(stream: &mut TcpStream, port: u16) -> String {
    // Some services send banners immediately, others need a prompt
    let probe = match port {
        80 | 8080 | 8443 | 443 => {
            Some(b"HEAD / HTTP/1.0\r\nHost: target\r\n\r\n".as_slice())
        }
        _ => None,
    };

    if let Some(data) = probe {
        let _ = stream.write_all(data).await;
    }

    let mut buf = vec![0u8; 1024];
    match tokio::time::timeout(Duration::from_secs(2), stream.read(&mut buf)).await {
        Ok(Ok(n)) if n > 0 => {
            let banner = String::from_utf8_lossy(&buf[..n])
                .trim()
                .chars()
                .take(200)
                .collect::<String>();
            // Clean non-printable characters
            banner.chars().filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()).collect()
        }
        _ => String::new(),
    }
}
