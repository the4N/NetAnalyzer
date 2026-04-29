// ─────────────────────────────────────────────────────────────
// NetAnalyzer - Windows WiFi Scanner (netsh wlan)
// ─────────────────────────────────────────────────────────────

use super::WifiNetwork;
use std::process::Command;

pub fn is_available() -> bool {
    Command::new("netsh")
        .args(["wlan", "show", "interfaces"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn scan() -> Result<Vec<WifiNetwork>, String> {
    let output = Command::new("netsh")
        .args(["wlan", "show", "networks", "mode=bssid"])
        .output()
        .map_err(|e| format!("Failed to execute netsh: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("netsh failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_netsh_output(&stdout)
}

fn parse_netsh_output(output: &str) -> Result<Vec<WifiNetwork>, String> {
    let mut networks = Vec::new();
    let mut current_ssid = String::new();
    let mut current_network_type = String::new();
    let mut current_security = String::new();

    for line in output.lines() {
        let line = line.trim();

        // SSID line (not BSSID)
        if (line.starts_with("SSID") && !line.starts_with("BSSID"))
            || (line.contains("SSID") && !line.contains("BSSID") && line.contains(':'))
        {
            if let Some(val) = extract_value(line) {
                if !val.is_empty() {
                    current_ssid = val;
                }
            }
        }

        // Network type
        if line.contains("Network type") || line.contains("ネットワークの種類") || line.contains("Type de réseau") {
            if let Some(val) = extract_value(line) {
                current_network_type = val;
            }
        }

        // Authentication / Security
        if line.contains("Authentication") || line.contains("認証") || line.contains("Authentification") {
            if let Some(val) = extract_value(line) {
                current_security = val;
            }
        }

        // BSSID line - start of a network entry
        if line.starts_with("BSSID") || (line.contains("BSSID") && line.contains(':')) {
            if let Some(val) = extract_value(line) {
                let bssid = val.trim().to_string();
                if bssid.contains(':') || bssid.contains('-') {
                    // We'll fill in signal/channel on subsequent lines
                    // For now, create a partial entry
                    let mut network = WifiNetwork {
                        ssid: current_ssid.clone(),
                        bssid,
                        signal_strength: -90,
                        signal_percent: 0,
                        channel: 0,
                        frequency: 0,
                        security: current_security.clone(),
                        band: String::new(),
                        network_type: current_network_type.clone(),
                    };
                    networks.push(network);
                }
            }
        }

        // Signal strength (percentage)
        if line.contains("Signal") || line.contains("シグナル") || line.contains("信号") {
            if let Some(val) = extract_value(line) {
                if let Some(percent) = val.trim_end_matches('%').trim().parse::<u32>().ok() {
                    if let Some(net) = networks.last_mut() {
                        net.signal_percent = percent;
                        net.signal_strength = WifiNetwork::percent_to_dbm(percent);
                    }
                }
            }
        }

        // Channel
        if line.contains("Channel") || line.contains("チャネル") || line.contains("Canal") {
            // Avoid "Radio type" lines
            if !line.contains("Radio") && !line.contains("radio") {
                if let Some(val) = extract_value(line) {
                    if let Ok(ch) = val.trim().parse::<u32>() {
                        if let Some(net) = networks.last_mut() {
                            net.channel = ch;
                            net.band = WifiNetwork::band_from_channel(ch);
                            net.frequency = WifiNetwork::freq_from_channel(ch);
                        }
                    }
                }
            }
        }
    }

    Ok(networks)
}

fn extract_value(line: &str) -> Option<String> {
    // Handle "Key : Value" format
    if let Some(pos) = line.find(':') {
        let val = line[pos + 1..].trim().to_string();
        Some(val)
    } else {
        None
    }
}
