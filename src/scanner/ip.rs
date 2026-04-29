// ─────────────────────────────────────────────────────────────
// NetAnalyzer - IP Scanner Engine
// ─────────────────────────────────────────────────────────────

use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::net::TcpStream;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpScanResult {
    pub ip: String,
    pub hostname: String,
    pub latency_ms: f64,
    pub is_alive: bool,
    pub method: String,
}

#[derive(Debug, Clone)]
pub enum ScanProgress {
    Update { scanned: usize, total: usize },
    Found(IpScanResult),
    Done,
    Error(String),
}

/// Scan a list of IPs using TCP connect (port 80/443) as a ping method.
/// This works without admin privileges on all platforms.
pub async fn scan_ips(
    ips: Vec<Ipv4Addr>,
    timeout: Duration,
    max_workers: usize,
    tx: mpsc::UnboundedSender<ScanProgress>,
) {
    let total = ips.len();
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_workers));
    let scanned = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let mut handles = Vec::new();

    for ip in ips {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let tx = tx.clone();
        let scanned = scanned.clone();

        let handle = tokio::spawn(async move {
            let result = ping_host(ip, timeout).await;
            let count = scanned.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;

            let _ = tx.send(ScanProgress::Update {
                scanned: count,
                total,
            });

            if result.is_alive {
                let _ = tx.send(ScanProgress::Found(result));
            }

            drop(permit);
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let _ = tx.send(ScanProgress::Done);
}

/// Try to ping a host using multiple methods
async fn ping_host(ip: Ipv4Addr, timeout: Duration) -> IpScanResult {
    let ip_str = ip.to_string();

    // Try ICMP ping first (requires admin on some systems)
    match try_icmp_ping(ip, timeout).await {
        Some(latency) => {
            let hostname = resolve_hostname(ip).await;
            return IpScanResult {
                ip: ip_str,
                hostname,
                latency_ms: latency,
                is_alive: true,
                method: "ICMP".to_string(),
            };
        }
        None => {}
    }

    // Fallback: TCP connect scan on common ports
    let ports = [80, 443, 22, 445, 139, 3389, 8080];
    for port in ports {
        match try_tcp_ping(ip, port, timeout).await {
            Some(latency) => {
                let hostname = resolve_hostname(ip).await;
                return IpScanResult {
                    ip: ip_str,
                    hostname,
                    latency_ms: latency,
                    is_alive: true,
                    method: format!("TCP:{}", port),
                };
            }
            None => continue,
        }
    }

    IpScanResult {
        ip: ip_str,
        hostname: String::new(),
        latency_ms: 0.0,
        is_alive: false,
        method: "N/A".to_string(),
    }
}

/// Attempt ICMP ping using surge-ping
async fn try_icmp_ping(ip: Ipv4Addr, timeout: Duration) -> Option<f64> {
    use surge_ping::{Client, Config, PingIdentifier, PingSequence, ICMP};

    let config = Config::builder().build();
    let client = match Client::new(&config) {
        Ok(c) => c,
        Err(_) => return None, // Likely needs admin privileges
    };

    let mut pinger = client.pinger(IpAddr::V4(ip), PingIdentifier(rand::random())).await;
    pinger.timeout(timeout);

    let payload = vec![0u8; 56];
    match pinger.ping(PingSequence(0), &payload).await {
        Ok((_, dur)) => Some(dur.as_secs_f64() * 1000.0),
        Err(_) => None,
    }
}

/// Attempt TCP connect as a ping method
async fn try_tcp_ping(ip: Ipv4Addr, port: u16, timeout: Duration) -> Option<f64> {
    let addr = format!("{}:{}", ip, port);
    let start = std::time::Instant::now();

    match tokio::time::timeout(timeout, TcpStream::connect(&addr)).await {
        Ok(Ok(_stream)) => {
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            Some(elapsed)
        }
        _ => None,
    }
}

/// Resolve hostname via reverse DNS
async fn resolve_hostname(ip: Ipv4Addr) -> String {
    tokio::task::spawn_blocking(move || {
        match dns_lookup::lookup_addr(&IpAddr::V4(ip)) {
            Ok(host) => host,
            Err(_) => String::new(),
        }
    })
    .await
    .unwrap_or_default()
}
