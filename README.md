# ⚡ NetAnalyzer

A cross-platform native desktop application for network analysis, built in **Rust** with **egui/eframe** GPU-rendered GUI.

## Features

| Feature | Description |
|---------|-------------|
| **IP Scanner** | ICMP/TCP ping sweep across IP ranges (CIDR notation) |
| **Port Scanner** | TCP connect scan with service detection and banner grabbing |
| **Speed Test** | Download/upload speed and latency measurement |
| **WiFi Scanner** | Discover nearby WiFi networks with signal, security, and band info |
| **Channel Analyzer** | Analyze WiFi channel congestion and get optimal channel recommendations |
| **Export Data** | Export scan results to JSON for reporting and external analysis |

## Screenshots

> 📸 *Screenshots coming soon!*

## Quick Start

### Pre-built Installers (Recommended)
You can download native installers from the **[GitHub Releases](../../releases)** page:
- **Windows**: `.msi` (x86_64, ARM64)
- **macOS**: `.dmg` (Intel, Apple Silicon M1/M2/M3)
- **Linux**: `.deb` (x86_64, ARM64)

### Prerequisites (For Building from Source)
- **Rust** (stable toolchain): Install via [rustup.rs](https://rustup.rs)
- **Windows**: No additional dependencies (WebView2 not needed — this is native GPU rendering)
- **Linux**: Install GPU/windowing development libraries:
  ```bash
  sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
  ```
- **macOS**: Xcode Command Line Tools: `xcode-select --install`

### Build & Run

```bash
# Debug mode (faster compilation)
cargo run

# Release mode (optimized, smaller binary)
cargo build --release
# Binary will be at: target/release/netanalyzer(.exe)
```

### Run Tests

```bash
cargo test
```

## Architecture

```
src/
├── main.rs          # Entry point
├── app.rs           # Main app struct, routing, state management
├── theme.rs         # Custom dark theme & styling
├── ui/              # GUI views
│   ├── sidebar.rs       # Navigation
│   ├── dashboard.rs     # Overview page
│   ├── ip_scanner.rs    # IP scanning page
│   ├── port_scanner.rs  # Port scanning page
│   ├── speed_test.rs    # Speed test page
│   ├── wifi_scanner.rs  # WiFi scanner page
│   ├── channel_analyzer.rs  # Channel analysis page
│   └── widgets/         # Custom widgets (gauge, progress bar, badges)
├── scanner/         # Network scanning engines
│   ├── ip.rs            # ICMP/TCP ping engine
│   ├── port.rs          # TCP port scanner
│   └── services.rs      # Port-to-service mapping
├── speed/           # Speed test engine
│   └── http_test.rs     # HTTP-based speed measurement
├── wifi/            # WiFi scanning (per-platform)
│   ├── windows.rs       # Windows (netsh wlan)
│   ├── linux.rs         # Linux (nmcli/iwlist)
│   ├── macos.rs         # macOS (system_profiler/airport)
│   └── channel.rs       # Channel congestion analysis
└── utils/           # Utilities
    └── network.rs       # CIDR parsing, port parsing, formatting
```

## CI/CD Pipeline

The project uses a powerful GitHub Actions workflow (`.github/workflows/release.yml`) that automatically builds native installers and portable binaries on every push to `main` or new `v*` tag:

- **Windows**: `.exe` and `.msi` (Using WiX Toolset)
- **Linux**: Portable `.tar.gz` and `.deb` (Using cargo-deb)
- **macOS**: Portable `.tar.gz` and `.dmg` (Using cargo-bundle and hdiutil)

**Supported Architectures:**
- `x86_64` (Intel/AMD) for all platforms
- `aarch64` (ARM64 / Apple Silicon / Snapdragon / RasPi) for all platforms

## Permissions

Some features require elevated privileges:
- **ICMP Ping**: Requires admin/root on most systems (falls back to TCP ping automatically)
- **WiFi Scanning**: May require admin/root depending on OS
- **Port Scanning**: Works without elevation (TCP connect scan)

## Tech Stack

- **Language**: Rust
- **GUI**: egui + eframe (GPU-rendered, immediate mode)
- **Async**: tokio (multi-threaded runtime)
- **Networking**: surge-ping, tokio::net, reqwest
- **Charts**: egui_plot
- **Serialization**: serde_json

## License

MIT
