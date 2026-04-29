// ─────────────────────────────────────────────────────────────
// NetAnalyzer - macOS WiFi Scanner (system_profiler / airport)
// ─────────────────────────────────────────────────────────────

use super::WifiNetwork;
use std::process::Command;

pub fn is_available() -> bool {
    Command::new("system_profiler")
        .args(["SPAirPortDataType"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn scan() -> Result<Vec<WifiNetwork>, String> {
    // Try airport utility first (deprecated but still works on some versions)
    if let Ok(result) = scan_airport() {
        if !result.is_empty() {
            return Ok(result);
        }
    }

    // Fallback: system_profiler
    scan_system_profiler()
}

fn scan_airport() -> Result<Vec<WifiNetwork>, String> {
    let airport_path = "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport";

    let output = Command::new(airport_path)
        .args(["-s"])
        .output()
        .map_err(|e| format!("airport command failed: {}", e))?;

    if !output.status.success() {
        return Err("airport -s failed".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut networks = Vec::new();

    for (i, line) in stdout.lines().enumerate() {
        if i == 0 { continue; } // Skip header

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 7 {
            // Format: SSID BSSID RSSI CHANNEL HT CC SECURITY
            let ssid = parts[0].to_string();
            let bssid = parts[1].to_string();
            let rssi: i32 = parts[2].parse().unwrap_or(-90);
            let channel: u32 = parts[3].split(',').next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let security = parts[6..].join(" ");

            networks.push(WifiNetwork {
                ssid,
                bssid,
                signal_strength: rssi,
                signal_percent: WifiNetwork::dbm_to_percent(rssi),
                channel,
                frequency: WifiNetwork::freq_from_channel(channel),
                security,
                band: WifiNetwork::band_from_channel(channel),
                network_type: "Infrastructure".to_string(),
            });
        }
    }

    Ok(networks)
}

fn scan_system_profiler() -> Result<Vec<WifiNetwork>, String> {
    let output = Command::new("system_profiler")
        .args(["SPAirPortDataType", "-json"])
        .output()
        .map_err(|e| format!("system_profiler failed: {}", e))?;

    if !output.status.success() {
        return Err("system_profiler failed".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse JSON output
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse system_profiler JSON: {}", e))?;

    let mut networks = Vec::new();

    // Navigate the JSON structure to find network info
    if let Some(airport_data) = json.get("SPAirPortDataType") {
        if let Some(arr) = airport_data.as_array() {
            for item in arr {
                if let Some(interfaces) = item.get("spairport_airport_interfaces") {
                    if let Some(iface_arr) = interfaces.as_array() {
                        for iface in iface_arr {
                            if let Some(other_networks) = iface.get("spairport_airport_other_local_wireless_networks") {
                                if let Some(net_arr) = other_networks.as_array() {
                                    for net in net_arr {
                                        let ssid = net.get("_name")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("Unknown")
                                            .to_string();

                                        let channel = net.get("spairport_network_channel")
                                            .and_then(|v| v.as_str())
                                            .and_then(|s| s.split(' ').next())
                                            .and_then(|s| s.parse::<u32>().ok())
                                            .unwrap_or(0);

                                        let rssi = net.get("spairport_signal_noise")
                                            .and_then(|v| v.as_str())
                                            .and_then(|s| s.split('/').next())
                                            .and_then(|s| s.trim().parse::<i32>().ok())
                                            .unwrap_or(-90);

                                        let security = net.get("spairport_security_mode")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("Unknown")
                                            .to_string();

                                        networks.push(WifiNetwork {
                                            ssid,
                                            bssid: String::new(),
                                            signal_strength: rssi,
                                            signal_percent: WifiNetwork::dbm_to_percent(rssi),
                                            channel,
                                            frequency: WifiNetwork::freq_from_channel(channel),
                                            security,
                                            band: WifiNetwork::band_from_channel(channel),
                                            network_type: "Infrastructure".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(networks)
}
