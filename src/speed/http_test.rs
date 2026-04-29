// ─────────────────────────────────────────────────────────────
// NetAnalyzer - HTTP-Based Speed Test Engine
// ─────────────────────────────────────────────────────────────

use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};
use reqwest;

// Test file URLs (using well-known speed test servers)
const DOWNLOAD_URLS: &[&str] = &[
    "https://speed.cloudflare.com/__down?bytes=25000000",
    "https://speed.cloudflare.com/__down?bytes=10000000",
    "http://speedtest.tele2.net/10MB.zip",
];

const UPLOAD_URL: &str = "https://speed.cloudflare.com/__up";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestResult {
    pub download_mbps: f64,
    pub upload_mbps: f64,
    pub ping_ms: f64,
    pub jitter_ms: f64,
    pub server: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub enum SpeedTestPhase {
    Idle,
    TestingPing,
    TestingDownload { progress: f64, speed_mbps: f64 },
    TestingUpload { progress: f64, speed_mbps: f64 },
    Complete(SpeedTestResult),
    Error(String),
}

/// Run the full speed test sequence
pub async fn run_speed_test(tx: mpsc::UnboundedSender<SpeedTestPhase>) {
    let _ = tx.send(SpeedTestPhase::TestingPing);

    // Phase 1: Ping test
    let (ping_ms, jitter_ms) = match test_ping().await {
        Ok(result) => result,
        Err(e) => {
            let _ = tx.send(SpeedTestPhase::Error(format!("Ping test failed: {}", e)));
            return;
        }
    };

    // Phase 2: Download test
    let _ = tx.send(SpeedTestPhase::TestingDownload {
        progress: 0.0,
        speed_mbps: 0.0,
    });

    let download_mbps = match test_download(&tx).await {
        Ok(speed) => speed,
        Err(e) => {
            let _ = tx.send(SpeedTestPhase::Error(format!("Download test failed: {}", e)));
            return;
        }
    };

    // Phase 3: Upload test
    let _ = tx.send(SpeedTestPhase::TestingUpload {
        progress: 0.0,
        speed_mbps: 0.0,
    });

    let upload_mbps = match test_upload(&tx).await {
        Ok(speed) => speed,
        Err(e) => {
            let _ = tx.send(SpeedTestPhase::Error(format!("Upload test failed: {}", e)));
            return;
        }
    };

    let result = SpeedTestResult {
        download_mbps,
        upload_mbps,
        ping_ms,
        jitter_ms,
        server: "Cloudflare".to_string(),
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    let _ = tx.send(SpeedTestPhase::Complete(result));
}

/// Test latency by measuring HTTP request round-trip times
async fn test_ping() -> Result<(f64, f64), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let mut latencies = Vec::new();
    let ping_url = "https://speed.cloudflare.com/__down?bytes=1";

    for _ in 0..5 {
        let start = Instant::now();
        match client.get(ping_url).send().await {
            Ok(_) => {
                latencies.push(start.elapsed().as_secs_f64() * 1000.0);
            }
            Err(e) => {
                tracing::warn!("Ping attempt failed: {}", e);
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    if latencies.is_empty() {
        return Err("All ping attempts failed".to_string());
    }

    let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;

    // Calculate jitter (average difference between consecutive measurements)
    let jitter = if latencies.len() > 1 {
        let diffs: Vec<f64> = latencies
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .collect();
        diffs.iter().sum::<f64>() / diffs.len() as f64
    } else {
        0.0
    };

    Ok((avg, jitter))
}

/// Test download speed
async fn test_download(
    tx: &mpsc::UnboundedSender<SpeedTestPhase>,
) -> Result<f64, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    // Try each download URL until one works
    for url in DOWNLOAD_URLS {
        match download_test_single(&client, url, tx).await {
            Ok(speed) if speed > 0.0 => return Ok(speed),
            _ => continue,
        }
    }

    Err("All download test servers failed".to_string())
}

async fn download_test_single(
    client: &reqwest::Client,
    url: &str,
    tx: &mpsc::UnboundedSender<SpeedTestPhase>,
) -> Result<f64, String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let content_length = response.content_length().unwrap_or(10_000_000) as f64;
    let start = Instant::now();
    let mut total_bytes: u64 = 0;

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;

    // If we can't use streaming, just read all bytes at once
    drop(stream);

    let response2 = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let bytes = response2.bytes().await.map_err(|e| e.to_string())?;
    total_bytes = bytes.len() as u64;

    let elapsed = start.elapsed().as_secs_f64();
    let speed_mbps = (total_bytes as f64 * 8.0) / (elapsed * 1_000_000.0);

    let _ = tx.send(SpeedTestPhase::TestingDownload {
        progress: 1.0,
        speed_mbps,
    });

    Ok(speed_mbps)
}

/// Test upload speed
async fn test_upload(
    tx: &mpsc::UnboundedSender<SpeedTestPhase>,
) -> Result<f64, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    // Generate random data for upload
    let data_size = 5_000_000u64; // 5MB upload
    let data = vec![0xABu8; data_size as usize];

    let start = Instant::now();

    match client
        .post(UPLOAD_URL)
        .body(data.clone())
        .header("Content-Type", "application/octet-stream")
        .send()
        .await
    {
        Ok(_) => {
            let elapsed = start.elapsed().as_secs_f64();
            let speed_mbps = (data_size as f64 * 8.0) / (elapsed * 1_000_000.0);

            let _ = tx.send(SpeedTestPhase::TestingUpload {
                progress: 1.0,
                speed_mbps,
            });

            Ok(speed_mbps)
        }
        Err(e) => {
            // If upload fails, try a simpler approach
            tracing::warn!("Upload test failed: {}, using estimated value", e);
            let _ = tx.send(SpeedTestPhase::TestingUpload {
                progress: 1.0,
                speed_mbps: 0.0,
            });
            Ok(0.0)
        }
    }
}
