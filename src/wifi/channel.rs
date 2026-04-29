// ─────────────────────────────────────────────────────────────
// NetAnalyzer - WiFi Channel Analyzer
// ─────────────────────────────────────────────────────────────

use super::WifiNetwork;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub channel: u32,
    pub frequency: u32,
    pub network_count: usize,
    pub networks: Vec<String>,       // SSIDs on this channel
    pub avg_signal: f64,
    pub congestion: CongestionLevel,
    pub is_recommended: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CongestionLevel {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for CongestionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CongestionLevel::Low => write!(f, "Low"),
            CongestionLevel::Medium => write!(f, "Medium"),
            CongestionLevel::High => write!(f, "High"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelAnalysis {
    pub channels_2g: Vec<ChannelInfo>,
    pub channels_5g: Vec<ChannelInfo>,
    pub recommended_2g: u32,
    pub recommended_5g: u32,
}

/// Analyze WiFi channels from scan results
pub fn analyze_channels(networks: &[WifiNetwork]) -> ChannelAnalysis {
    // 2.4 GHz non-overlapping channels: 1, 6, 11
    // All 2.4 GHz channels: 1-14
    let channels_2g = analyze_band(networks, 1..=14);

    // 5 GHz common channels
    let channels_5g_list = vec![
        36, 40, 44, 48, 52, 56, 60, 64,
        100, 104, 108, 112, 116, 120, 124, 128, 132, 136, 140, 144,
        149, 153, 157, 161, 165,
    ];
    let channels_5g = analyze_band_list(networks, &channels_5g_list);

    // Find recommended channels (non-overlapping with lowest congestion)
    let recommended_2g = find_best_channel(&channels_2g, &[1, 6, 11]);
    let recommended_5g = find_best_channel(&channels_5g, &channels_5g_list);

    // Mark recommended channels
    let mut channels_2g = channels_2g;
    let mut channels_5g = channels_5g;

    for ch in &mut channels_2g {
        ch.is_recommended = ch.channel == recommended_2g;
    }
    for ch in &mut channels_5g {
        ch.is_recommended = ch.channel == recommended_5g;
    }

    ChannelAnalysis {
        channels_2g,
        channels_5g,
        recommended_2g,
        recommended_5g,
    }
}

fn analyze_band(
    networks: &[WifiNetwork],
    channel_range: std::ops::RangeInclusive<u32>,
) -> Vec<ChannelInfo> {
    let channels: Vec<u32> = channel_range.collect();
    analyze_band_list(networks, &channels)
}

fn analyze_band_list(
    networks: &[WifiNetwork],
    channels: &[u32],
) -> Vec<ChannelInfo> {
    channels
        .iter()
        .map(|&ch| {
            let on_channel: Vec<&WifiNetwork> = networks
                .iter()
                .filter(|n| n.channel == ch)
                .collect();

            let network_count = on_channel.len();
            let ssids: Vec<String> = on_channel.iter().map(|n| n.ssid.clone()).collect();

            let avg_signal = if network_count > 0 {
                on_channel.iter().map(|n| n.signal_strength as f64).sum::<f64>()
                    / network_count as f64
            } else {
                -100.0
            };

            // Also count networks on adjacent channels that cause interference (2.4 GHz)
            let interference_count = if ch <= 14 {
                networks
                    .iter()
                    .filter(|n| {
                        n.channel <= 14
                            && n.channel != ch
                            && (n.channel as i32 - ch as i32).unsigned_abs() <= 4
                    })
                    .count()
            } else {
                0
            };

            let total_congestion = network_count + interference_count / 2;
            let congestion = match total_congestion {
                0..=2 => CongestionLevel::Low,
                3..=5 => CongestionLevel::Medium,
                _ => CongestionLevel::High,
            };

            ChannelInfo {
                channel: ch,
                frequency: WifiNetwork::freq_from_channel(ch),
                network_count,
                networks: ssids,
                avg_signal,
                congestion,
                is_recommended: false,
            }
        })
        .collect()
}

fn find_best_channel(channels: &[ChannelInfo], preferred: &[u32]) -> u32 {
    channels
        .iter()
        .filter(|ch| preferred.contains(&ch.channel))
        .min_by_key(|ch| ch.network_count)
        .map(|ch| ch.channel)
        .unwrap_or(preferred.first().copied().unwrap_or(1))
}
