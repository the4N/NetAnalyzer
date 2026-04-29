// ─────────────────────────────────────────────────────────────
// NetAnalyzer - Linux WiFi Scanner (nmcli / iwlist)
// ─────────────────────────────────────────────────────────────

use super::WifiNetwork;
use std::process::Command;

pub fn is_available() -> bool {
    // Check if nmcli or iwlist is available
    Command::new("which")
        .arg("nmcli")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || Command::new("which")
            .arg("iwlist")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
}

pub fn scan() -> Result<Vec<WifiNetwork>, String> {
    // Try nmcli first (modern, structured output)
    if let Ok(result) = scan_nmcli() {
        if !result.is_empty() {
            return Ok(result);
        }
    }

    // Fallback to iwlist
    scan_iwlist()
}

fn scan_nmcli() -> Result<Vec<WifiNetwork>, String> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "SSID,BSSID,CHAN,FREQ,SIGNAL,SECURITY,MODE", "dev", "wifi", "list", "--rescan", "yes"])
        .output()
        .map_err(|e| format!("Failed to execute nmcli: {}", e))?;

    if !output.status.success() {
        return Err("nmcli failed".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut networks = Vec::new();

    for line in stdout.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() >= 6 {
            let ssid = fields[0].replace("\\:", ":").trim().to_string();
            let bssid = fields[1].trim().to_string();
            let channel: u32 = fields[2].trim().parse().unwrap_or(0);
            let freq_str = fields[3].trim();
            let signal: u32 = fields[4].trim().parse().unwrap_or(0);
            let security = fields[5].trim().to_string();
            let mode = if fields.len() > 6 { fields[6].trim().to_string() } else { String::new() };

            let frequency = freq_str
                .chars()
                .filter(|c| c.is_digit(10))
                .collect::<String>()
                .parse::<u32>()
                .unwrap_or(WifiNetwork::freq_from_channel(channel));

            networks.push(WifiNetwork {
                ssid,
                bssid,
                signal_strength: WifiNetwork::percent_to_dbm(signal),
                signal_percent: signal,
                channel,
                frequency,
                security,
                band: WifiNetwork::band_from_channel(channel),
                network_type: mode,
            });
        }
    }

    Ok(networks)
}

fn scan_iwlist() -> Result<Vec<WifiNetwork>, String> {
    // Try to find the wireless interface
    let iface = find_wireless_interface().unwrap_or_else(|| "wlan0".to_string());

    let output = Command::new("sudo")
        .args(["iwlist", &iface, "scan"])
        .output()
        .or_else(|_| {
            Command::new("iwlist")
                .args([&iface, "scan"])
                .output()
        })
        .map_err(|e| format!("Failed to execute iwlist: {}", e))?;

    if !output.status.success() {
        return Err("iwlist scan failed (may need sudo)".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_iwlist_output(&stdout)
}

fn find_wireless_interface() -> Option<String> {
    let output = Command::new("iw")
        .args(["dev"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Interface") {
            return Some(line.split_whitespace().nth(1)?.to_string());
        }
    }
    None
}

fn parse_iwlist_output(output: &str) -> Result<Vec<WifiNetwork>, String> {
    let mut networks = Vec::new();
    let mut current: Option<WifiNetwork> = None;

    for line in output.lines() {
        let line = line.trim();

        if line.contains("Cell") && line.contains("Address:") {
            if let Some(net) = current.take() {
                networks.push(net);
            }
            let bssid = line
                .split("Address:")
                .nth(1)
                .unwrap_or("")
                .trim()
                .to_string();

            current = Some(WifiNetwork {
                ssid: String::new(),
                bssid,
                signal_strength: -90,
                signal_percent: 0,
                channel: 0,
                frequency: 0,
                security: String::new(),
                band: String::new(),
                network_type: String::new(),
            });
        }

        if let Some(ref mut net) = current {
            if line.starts_with("ESSID:") {
                net.ssid = line
                    .split('"')
                    .nth(1)
                    .unwrap_or("")
                    .to_string();
            }

            if line.contains("Channel:") {
                if let Some(ch_str) = line.split("Channel:").nth(1) {
                    if let Ok(ch) = ch_str.trim().parse::<u32>() {
                        net.channel = ch;
                        net.band = WifiNetwork::band_from_channel(ch);
                    }
                }
            }

            if line.contains("Frequency:") {
                if let Some(freq_str) = line.split("Frequency:").nth(1) {
                    let freq: String = freq_str.chars().take_while(|c| c.is_digit(10) || *c == '.').collect();
                    if let Ok(ghz) = freq.parse::<f64>() {
                        net.frequency = (ghz * 1000.0) as u32;
                    }
                }
            }

            if line.contains("Signal level=") || line.contains("Signal level:") {
                let sep = if line.contains("Signal level=") { "Signal level=" } else { "Signal level:" };
                if let Some(sig_str) = line.split(sep).nth(1) {
                    let sig: String = sig_str.chars().take_while(|c| c.is_digit(10) || *c == '-').collect();
                    if let Ok(dbm) = sig.parse::<i32>() {
                        net.signal_strength = dbm;
                        net.signal_percent = WifiNetwork::dbm_to_percent(dbm);
                    }
                }
            }

            if line.contains("IE:") {
                if line.contains("WPA2") {
                    net.security = "WPA2".to_string();
                } else if line.contains("WPA") {
                    net.security = "WPA".to_string();
                }
            }
        }
    }

    if let Some(net) = current {
        networks.push(net);
    }

    Ok(networks)
}
