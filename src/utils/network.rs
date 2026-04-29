// ─────────────────────────────────────────────────────────────
// NetAnalyzer - Network Utilities
// ─────────────────────────────────────────────────────────────

use std::net::Ipv4Addr;

/// Parse a CIDR notation string (e.g., "192.168.1.0/24") into a list of IP addresses.
pub fn parse_cidr(cidr: &str) -> Result<Vec<Ipv4Addr>, String> {
    let parts: Vec<&str> = cidr.trim().split('/').collect();
    if parts.len() != 2 {
        return Err("Invalid CIDR format. Use: x.x.x.x/xx".to_string());
    }

    let base_ip: Ipv4Addr = parts[0]
        .parse()
        .map_err(|_| "Invalid IP address".to_string())?;

    let prefix_len: u32 = parts[1]
        .parse()
        .map_err(|_| "Invalid prefix length".to_string())?;

    if prefix_len > 32 {
        return Err("Prefix length must be 0-32".to_string());
    }

    if prefix_len < 16 {
        return Err("Prefix too large (would scan too many hosts). Use /16 or smaller.".to_string());
    }

    let ip_u32 = u32::from(base_ip);
    let mask = if prefix_len == 0 {
        0u32
    } else {
        !((1u32 << (32 - prefix_len)) - 1)
    };

    let network = ip_u32 & mask;
    let broadcast = network | !mask;
    let host_count = broadcast - network;

    // Skip network and broadcast addresses for /31 and larger
    let (start, end) = if host_count > 1 {
        (network + 1, broadcast - 1) // Skip network & broadcast
    } else {
        (network, broadcast)
    };

    let mut ips = Vec::new();
    for i in start..=end {
        ips.push(Ipv4Addr::from(i));
    }

    Ok(ips)
}

/// Parse port range strings like "1-1024", "22,80,443", "80"
pub fn parse_ports(port_str: &str) -> Result<Vec<u16>, String> {
    let mut ports = Vec::new();
    let port_str = port_str.trim();

    for part in port_str.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            if range.len() != 2 {
                return Err(format!("Invalid port range: {}", part));
            }
            let start: u16 = range[0]
                .trim()
                .parse()
                .map_err(|_| format!("Invalid port number: {}", range[0]))?;
            let end: u16 = range[1]
                .trim()
                .parse()
                .map_err(|_| format!("Invalid port number: {}", range[1]))?;
            if start > end {
                return Err(format!("Invalid range: {} > {}", start, end));
            }
            for p in start..=end {
                ports.push(p);
            }
        } else {
            let port: u16 = part
                .parse()
                .map_err(|_| format!("Invalid port number: {}", part))?;
            ports.push(port);
        }
    }

    Ok(ports)
}

/// Get the top N most common ports
pub fn top_ports(n: usize) -> Vec<u16> {
    let common = vec![
        21, 22, 23, 25, 53, 80, 110, 111, 135, 139,
        143, 443, 445, 993, 995, 1723, 3306, 3389, 5432, 5900,
        5985, 5986, 6379, 8080, 8443, 8888, 9090, 9200, 27017,
        // Extended list
        20, 26, 37, 49, 69, 79, 81, 82, 83, 84, 85, 88, 106, 113,
        119, 123, 137, 138, 144, 161, 162, 179, 194, 199, 389, 427,
        444, 465, 500, 513, 514, 515, 520, 523, 548, 554, 587, 593,
        631, 636, 873, 902, 912, 990, 1024, 1025, 1026, 1027, 1028,
        1029, 1030, 1080, 1099, 1194, 1214, 1241, 1311, 1337, 1433,
        1434, 1521, 1720, 1900, 2000, 2049, 2082, 2083, 2086, 2087,
        2096, 2181, 2222, 2375, 2376, 3000, 3128, 3268, 3269, 3333,
        3478, 4000, 4443, 4444, 4567, 4711, 4848, 5000, 5001, 5050,
        5060, 5222, 5353, 5500, 5555, 5601, 5672, 5800, 5901, 6000,
        6001, 6443, 6666, 6667, 7000, 7001, 7002, 7199, 7443, 7474,
        8000, 8001, 8008, 8009, 8010, 8081, 8082, 8083, 8084, 8085,
    ];
    common.into_iter().take(n).collect()
}

/// Format bytes per second into human-readable speed
pub fn format_speed(bytes_per_sec: f64) -> String {
    let bits = bytes_per_sec * 8.0;
    if bits >= 1_000_000_000.0 {
        format!("{:.1} Gbps", bits / 1_000_000_000.0)
    } else if bits >= 1_000_000.0 {
        format!("{:.1} Mbps", bits / 1_000_000.0)
    } else if bits >= 1_000.0 {
        format!("{:.1} Kbps", bits / 1_000.0)
    } else {
        format!("{:.0} bps", bits)
    }
}

/// Format latency in milliseconds
pub fn format_latency(ms: f64) -> String {
    if ms < 1.0 {
        format!("{:.2} ms", ms)
    } else if ms < 100.0 {
        format!("{:.1} ms", ms)
    } else {
        format!("{:.0} ms", ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cidr_24() {
        let ips = parse_cidr("192.168.1.0/24").unwrap();
        assert_eq!(ips.len(), 254);
        assert_eq!(ips[0], Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(ips[253], Ipv4Addr::new(192, 168, 1, 254));
    }

    #[test]
    fn test_parse_ports_range() {
        let ports = parse_ports("1-5").unwrap();
        assert_eq!(ports, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_parse_ports_list() {
        let ports = parse_ports("22,80,443").unwrap();
        assert_eq!(ports, vec![22, 80, 443]);
    }

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(12_500_000.0), "100.0 Mbps");
    }
}
