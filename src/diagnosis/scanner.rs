use crate::diagnosis::{Diagnosis, DiagnosisReport, Status};
use crate::wp::WpCli;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::Result;
use sysinfo::System;

const CANDIDATE_LOG_PATHS: &[&str] = &[
    "/usr/local/lsws/logs/access.log",
    "/var/log/apache2/access.log",
    "/var/log/httpd/access_log",
    "/var/log/nginx/access.log",
];

const TAIL_LINES: usize = 5000;
const NOT_FOUND_THRESHOLD: usize = 20;
const BAD_PATH_MARKERS: &[&str] = &[
    "wp-config.php",
    "/.env",
    "/actuator/",
    "/v1/graphql",
    "/api/graphql",
    "/graphql",
    "/debug/vars",
    ".zip",
    ".sql",
    ".bak",
];

pub struct ScannerDiagnosis;

impl Diagnosis for ScannerDiagnosis {
    fn run(&self, _wp: &WpCli, _root: &Path) -> Result<DiagnosisReport> {
        println!("  Running Scanner/Bot Traffic Diagnosis...");
        let mut details = Vec::new();
        let mut status = Status::Ok;

        let log_path = Self::find_access_log();
        let log_path = match log_path {
            Some(p) => p,
            None => {
                details.push("Could not locate a web-server access log (checked common LiteSpeed/Apache/Nginx paths).".to_string());
                return Ok(DiagnosisReport {
                    module: "Scanner".to_string(),
                    status: Status::Warning,
                    message: "Access log not found; cannot check for bot/scanner traffic.".to_string(),
                    details,
                });
            }
        };

        let tail = match Self::tail_file(&log_path, TAIL_LINES) {
            Ok(t) => t,
            Err(e) => {
                details.push(format!("Found access log at {:?} but could not read it: {}", log_path, e));
                return Ok(DiagnosisReport {
                    module: "Scanner".to_string(),
                    status: Status::Warning,
                    message: "Access log unreadable.".to_string(),
                    details,
                });
            }
        };

        details.push(format!("Scanned last {} lines of {:?}.", TAIL_LINES, log_path));

        let stats = Self::analyze_access_log(&tail);
        let fail2ban_active = Self::fail2ban_active();
        let scanner_detected = !stats.suspicious_ips.is_empty();

        if scanner_detected {
            let mut offenders: Vec<(&String, &IpStats)> = stats.suspicious_ips.iter().collect();
            offenders.sort_by(|a, b| b.1.total.cmp(&a.1.total));
            details.push(format!("{} IP(s) show scanner/bot-like behavior in the scanned window:", offenders.len()));
            for (ip, s) in offenders.into_iter().take(10) {
                details.push(format!(
                    " - {}: {} requests, {} 404s, {} known-bad-path hit(s)",
                    ip, s.total, s.not_found, s.bad_path
                ));
            }

            if fail2ban_active {
                status = Status::Warning;
                details.push("fail2ban is active — verify these IPs are being banned (fail2ban-client status <jail>).".to_string());
            } else {
                status = Status::Error;
                details.push("fail2ban is NOT installed/active. Scanner traffic is hitting the server unmitigated — install and configure fail2ban.".to_string());
            }
        } else {
            details.push("No scanner/bot-like IP behavior detected in the scanned window.".to_string());
        }

        if !fail2ban_active {
            details.push("fail2ban not detected on this host (no active fail2ban-client/service).".to_string());
        } else {
            details.push("fail2ban detected and active.".to_string());
        }

        Self::correlate_load(&stats, &mut status, &mut details);

        let message = if scanner_detected {
            "Scanner/bot traffic detected.".to_string()
        } else {
            "No scanner/bot traffic detected.".to_string()
        };

        Ok(DiagnosisReport {
            module: "Scanner".to_string(),
            status,
            message,
            details,
        })
    }
}

#[derive(Default)]
struct IpStats {
    total: usize,
    not_found: usize,
    bad_path: usize,
}

#[derive(Default)]
struct AccessLogStats {
    total_requests: usize,
    suspicious_ips: HashMap<String, IpStats>,
}

impl ScannerDiagnosis {
    fn find_access_log() -> Option<PathBuf> {
        for candidate in CANDIDATE_LOG_PATHS {
            let path = PathBuf::from(candidate);
            if path.exists() {
                return Some(path);
            }
        }
        None
    }

    fn tail_file(path: &Path, max_lines: usize) -> std::io::Result<String> {
        let contents = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = contents.lines().collect();
        let start = lines.len().saturating_sub(max_lines);
        Ok(lines[start..].join("\n"))
    }

    fn analyze_access_log(log_tail: &str) -> AccessLogStats {
        let mut per_ip: HashMap<String, IpStats> = HashMap::new();
        let mut total_requests = 0usize;

        for line in log_tail.lines() {
            if line.trim().is_empty() {
                continue;
            }
            total_requests += 1;

            let ip = match line.split_whitespace().next() {
                Some(ip) => ip.to_string(),
                None => continue,
            };

            let entry = per_ip.entry(ip).or_default();
            entry.total += 1;

            if line.contains("\" 404") {
                entry.not_found += 1;
            }

            if BAD_PATH_MARKERS.iter().any(|marker| line.contains(marker)) {
                entry.bad_path += 1;
            }
        }

        let suspicious_ips: HashMap<String, IpStats> = per_ip
            .into_iter()
            .filter(|(_, s)| s.not_found >= NOT_FOUND_THRESHOLD || s.bad_path >= 1)
            .collect();

        AccessLogStats {
            total_requests,
            suspicious_ips,
        }
    }

    fn fail2ban_active() -> bool {
        let has_client = Command::new("which")
            .arg("fail2ban-client")
            .output()
            .map(|o| o.status.success() && !o.stdout.is_empty())
            .unwrap_or(false);

        if !has_client {
            return false;
        }

        Command::new("systemctl")
            .args(["is-active", "fail2ban"])
            .output()
            .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "active")
            .unwrap_or(false)
    }

    fn correlate_load(stats: &AccessLogStats, status: &mut Status, details: &mut Vec<String>) {
        let mut sys = System::new_all();
        sys.refresh_all();

        let load = System::load_average();
        let cores = sys.cpus().len().max(1) as f64;
        let load_ratio = load.one / cores;

        details.push(format!(
            "Load average (1m): {:.2} across {} core(s) (ratio {:.2}).",
            load.one, cores as usize, load_ratio
        ));

        if load_ratio < 1.0 {
            return;
        }

        if stats.total_requests < 200 && stats.suspicious_ips.is_empty() {
            details.push(format!(
                "High CPU load (ratio {:.2}) but only {} requests in the scanned access-log window — load is likely NOT caused by visitor/bot traffic. Investigate with `ps aux --sort=-%cpu` and check for a runaway plugin/cron job.",
                load_ratio, stats.total_requests
            ));
            if *status == Status::Ok {
                *status = Status::Warning;
            }
        } else {
            details.push(format!(
                "High CPU load (ratio {:.2}) correlates with {} requests in the scanned window — traffic-driven load.",
                load_ratio, stats.total_requests
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_access_log_flags_high_404_ip() {
        let mut log = String::new();
        for _ in 0..25 {
            log.push_str("1.2.3.4 - - [12/Sep/2026:00:00:00 +0000] \"GET /random-path HTTP/1.1\" 404 0\n");
        }
        let stats = ScannerDiagnosis::analyze_access_log(&log);
        assert_eq!(stats.total_requests, 25);
        assert!(stats.suspicious_ips.contains_key("1.2.3.4"));
        assert_eq!(stats.suspicious_ips["1.2.3.4"].not_found, 25);
    }

    #[test]
    fn test_analyze_access_log_flags_single_bad_path_hit() {
        let log = "5.6.7.8 - - [12/Sep/2026:00:00:00 +0000] \"GET /wp-config.php~ HTTP/1.1\" 200 0";
        let stats = ScannerDiagnosis::analyze_access_log(log);
        assert!(stats.suspicious_ips.contains_key("5.6.7.8"));
        assert_eq!(stats.suspicious_ips["5.6.7.8"].bad_path, 1);
    }

    #[test]
    fn test_analyze_access_log_clean_traffic_not_flagged() {
        let log = "9.9.9.9 - - [12/Sep/2026:00:00:00 +0000] \"GET /index.php HTTP/1.1\" 200 512";
        let stats = ScannerDiagnosis::analyze_access_log(log);
        assert!(stats.suspicious_ips.is_empty());
        assert_eq!(stats.total_requests, 1);
    }
}
