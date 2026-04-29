// ─────────────────────────────────────────────────────────────
// NetAnalyzer - WiFi Scanner Module
// ─────────────────────────────────────────────────────────────

pub mod channel;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub bssid: String,
    pub signal_strength: i32,  // dBm (negative value, e.g., -45)
    pub signal_percent: u32,   // 0-100%
    pub channel: u32,
    pub frequency: u32,        // MHz
    pub security: String,
    pub band: String,          // "2.4 GHz" / "5 GHz" / "6 GHz"
    pub network_type: String,  // Infrastructure, Ad-hoc, etc.
}

impl WifiNetwork {
    /// Convert signal percentage to approximate dBm
    pub fn percent_to_dbm(percent: u32) -> i32 {
        // Approximate: 100% ≈ -30 dBm, 0% ≈ -90 dBm
        -90 + (percent as i32 * 60 / 100)
    }

    /// Convert dBm to percentage
    pub fn dbm_to_percent(dbm: i32) -> u32 {
        if dbm >= -30 { return 100; }
        if dbm <= -90 { return 0; }
        ((dbm + 90) * 100 / 60) as u32
    }

    /// Get band from channel number
    pub fn band_from_channel(ch: u32) -> String {
        if ch <= 14 {
            "2.4 GHz".to_string()
        } else if ch <= 177 {
            "5 GHz".to_string()
        } else {
            "6 GHz".to_string()
        }
    }

    /// Get approximate frequency from channel number
    pub fn freq_from_channel(ch: u32) -> u32 {
        match ch {
            1..=13 => 2407 + ch * 5,
            14 => 2484,
            36..=177 => 5000 + ch * 5,
            _ => 0,
        }
    }
}

/// Scan for WiFi networks using the platform-specific implementation
pub fn scan_wifi() -> Result<Vec<WifiNetwork>, String> {
    #[cfg(target_os = "windows")]
    return windows::scan();

    #[cfg(target_os = "linux")]
    return linux::scan();

    #[cfg(target_os = "macos")]
    return macos::scan();

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    return Err("WiFi scanning is not supported on this platform".to_string());
}

/// Check if WiFi scanning is available
pub fn is_wifi_available() -> bool {
    #[cfg(target_os = "windows")]
    return windows::is_available();

    #[cfg(target_os = "linux")]
    return linux::is_available();

    #[cfg(target_os = "macos")]
    return macos::is_available();

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    return false;
}
