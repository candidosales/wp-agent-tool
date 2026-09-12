use crate::diagnosis::{Diagnosis, DiagnosisReport, Status};
use crate::wp::WpCli;
use std::path::Path;
use anyhow::Result;

pub struct PerformanceDiagnosis;

impl Diagnosis for PerformanceDiagnosis {
    fn run(&self, wp: &WpCli, root: &Path) -> Result<DiagnosisReport> {
        println!("  Running Performance Diagnosis...");
        let mut details = Vec::new();
        let mut overall_status = Status::Ok;

        // 1. Check Autoloaded Options Size
        println!("    > Checking autoloaded options size...");

        let prefix = wp.run(&["config", "get", "table_prefix"], root).ok();
        match &prefix {
            Some(prefix) => {
                 let prefix = prefix.trim();
                 let query = format!("SELECT SUM(LENGTH(option_value)) FROM {}options WHERE autoload = 'yes'", prefix);
                 let size_result = wp.run(&["db", "query", &query, "--skip-column-names"], root)
                     .or_else(|_| wp.raw_query(root, &query));
                 match size_result {
                     Ok(size_str) => {
                         self.analyze_autoload_size(&size_str, &mut overall_status, &mut details);
                     }
                     Err(_) => details.push("Could not determine autoloaded options size (wp-cli and raw DB fallback both failed).".to_string()),
                 }
            }
            None => details.push("Could not determine table prefix.".to_string()),
        }

        // 2. Check Cron: flag DISABLE_WP_CRON with no real replacement, not just "command ran ok".
        println!("    > Checking cron events...");
        self.analyze_cron(wp, root, &mut overall_status, &mut details);

        // 3. PHP memory_limit / max_execution_time vs plugin count
        println!("    > Checking PHP limits...");
        self.analyze_php_limits(wp, root, &mut overall_status, &mut details);

        // 4. Object Cache
        println!("    > Checking object cache...");
        self.analyze_object_cache(wp, root, &mut details);

        Ok(DiagnosisReport {
            module: "Performance".to_string(),
            status: overall_status,
            message: "Performance Checked".to_string(),
            details,
        })
    }
}

impl PerformanceDiagnosis {
    fn analyze_autoload_size(&self, size_str: &str, status: &mut Status, details: &mut Vec<String>) {
        let size_bytes: u64 = size_str.trim().parse().unwrap_or(0);
        let size_mb = size_bytes as f64 / 1024.0 / 1024.0;
        details.push(format!("Autoloaded options size: {:.2} MB", size_mb));

        if size_mb > 1.0 {
            *status = Status::Warning;
            details.push("Warning: Autoloaded options size is high (> 1MB).".to_string());
        }
    }

    fn analyze_cron(&self, wp: &WpCli, root: &Path, status: &mut Status, details: &mut Vec<String>) {
        let disabled = wp
            .run(&["config", "get", "DISABLE_WP_CRON"], root)
            .map(|v| { let v = v.trim(); v == "1" || v.eq_ignore_ascii_case("true") })
            .unwrap_or(false);

        if !disabled {
            details.push("WP-Cron is enabled (pseudo-cron on page load).".to_string());
            return;
        }

        details.push("DISABLE_WP_CRON is set — WordPress will not trigger cron on page load.".to_string());

        if Self::system_cron_hits_wp_cron() {
            details.push("A system cron entry calling wp-cron.php was found — scheduled tasks should still run.".to_string());
        } else {
            *status = Status::Warning;
            details.push(
                "Warning: DISABLE_WP_CRON is set but no system cron entry hitting wp-cron.php was found in crontab/etc/cron.d. \
                 Scheduled tasks (plugin updates, security scans, cache warmup) will silently never run.".to_string(),
            );
        }
    }

    fn system_cron_hits_wp_cron() -> bool {
        let crontab_output = std::process::Command::new("crontab")
            .args(["-l"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        if crontab_output.contains("wp-cron.php") {
            return true;
        }

        let cron_d = Path::new("/etc/cron.d");
        if let Ok(entries) = std::fs::read_dir(cron_d) {
            for entry in entries.flatten() {
                if let Ok(contents) = std::fs::read_to_string(entry.path()) {
                    if contents.contains("wp-cron.php") {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn analyze_php_limits(&self, wp: &WpCli, root: &Path, status: &mut Status, details: &mut Vec<String>) {
        let active_plugin_count = wp
            .run(&["plugin", "list", "--status=active", "--field=name", "--format=csv"], root)
            .map(|s| s.lines().filter(|l| !l.trim().is_empty() && *l != "name").count())
            .unwrap_or(0);

        let memory_limit = wp
            .run(&["eval", "echo ini_get('memory_limit');"], root)
            .map(|s| s.trim().to_string())
            .ok();
        let max_execution_time = wp
            .run(&["eval", "echo ini_get('max_execution_time');"], root)
            .map(|s| s.trim().to_string())
            .ok();

        match &memory_limit {
            Some(limit) => {
                details.push(format!("PHP memory_limit: {}", limit));
                if let Some(mb) = Self::parse_php_size_mb(limit) {
                    let recommended_min = 128.0 + (active_plugin_count as f64 * 4.0);
                    if mb < recommended_min {
                        *status = Status::Warning;
                        details.push(format!(
                            "Warning: memory_limit ({:.0}M) is low for {} active plugins (suggest >= {:.0}M) — risk of fatal out-of-memory crashes under load.",
                            mb, active_plugin_count, recommended_min
                        ));
                    }
                }
            }
            None => details.push("Could not determine PHP memory_limit.".to_string()),
        }

        match &max_execution_time {
            Some(t) => {
                details.push(format!("PHP max_execution_time: {}s", t));
                if let Ok(secs) = t.parse::<i64>() {
                    if secs != 0 && secs < 60 {
                        *status = Status::Warning;
                        details.push("Warning: max_execution_time is under 60s — long-running requests (imports, scans, image processing) can be killed mid-execution.".to_string());
                    }
                }
            }
            None => details.push("Could not determine PHP max_execution_time.".to_string()),
        }
    }

    fn parse_php_size_mb(value: &str) -> Option<f64> {
        let value = value.trim();
        if value.is_empty() || value == "-1" {
            return None;
        }
        let (num_part, unit) = value.split_at(value.len() - 1);
        let num: f64 = num_part.parse().ok()?;
        match unit.to_uppercase().as_str() {
            "G" => Some(num * 1024.0),
            "M" => Some(num),
            "K" => Some(num / 1024.0),
            _ => value.parse::<f64>().ok().map(|b| b / 1024.0 / 1024.0),
        }
    }

    fn analyze_object_cache(&self, wp: &WpCli, root: &Path, details: &mut Vec<String>) {
        let mut found: Vec<String> = Vec::new();

        for (plugin, label) in [
            ("redis-cache", "Redis Object Cache plugin"),
            ("w3-total-cache", "W3 Total Cache"),
            ("litespeed-cache", "LiteSpeed Cache plugin (object cache)"),
        ] {
            if wp.run(&["plugin", "is-active", plugin], root).is_ok() {
                found.push(label.to_string());
            }
        }

        if root.join("wp-content/object-cache.php").exists() {
            found.push("object-cache.php drop-in present".to_string());
        }

        if Self::memcached_running() {
            found.push("Memcached process/service detected on the host".to_string());
        }

        if found.is_empty() {
            details.push("No persistent object cache detected (no Redis/W3TC/LiteSpeed plugin, no object-cache.php drop-in, no Memcached).".to_string());
        } else {
            details.push(format!("Persistent object cache indicators: {}.", found.join(", ")));
        }
    }

    fn memcached_running() -> bool {
        std::process::Command::new("pgrep")
            .arg("memcached")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_autoload_size_large() {
        let diagnosis = PerformanceDiagnosis;
        let mut status = Status::Ok;
        let mut details = Vec::new();
        // 2 MB in bytes
        let size_str = "2097152";

        diagnosis.analyze_autoload_size(size_str, &mut status, &mut details);

        assert_eq!(status, Status::Warning);
        assert!(details.iter().any(|d| d.contains("Autoloaded options size: 2.00 MB")));
    }

    #[test]
    fn test_analyze_autoload_size_small() {
        let diagnosis = PerformanceDiagnosis;
        let mut status = Status::Ok;
        let mut details = Vec::new();
        let size_str = "102400";

        diagnosis.analyze_autoload_size(size_str, &mut status, &mut details);

        assert_eq!(status, Status::Ok);
    }

    #[test]
    fn test_parse_php_size_mb_variants() {
        assert_eq!(PerformanceDiagnosis::parse_php_size_mb("256M"), Some(256.0));
        assert_eq!(PerformanceDiagnosis::parse_php_size_mb("1G"), Some(1024.0));
        assert_eq!(PerformanceDiagnosis::parse_php_size_mb("-1"), None);
    }
}
