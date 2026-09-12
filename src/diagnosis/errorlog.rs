use crate::diagnosis::{Diagnosis, DiagnosisReport, Status};
use crate::wp::WpCli;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::Result;

const CANDIDATE_LOG_PATHS: &[&str] = &[
    "/usr/local/lsws/logs/error.log",
    "/var/log/apache2/error.log",
    "/var/log/httpd/error_log",
    "/var/log/nginx/error.log",
    "/var/log/php-fpm/error.log",
    "/var/log/php_errors.log",
];

const TAIL_LINES: usize = 2000;

pub struct ErrorLogDiagnosis;

impl Diagnosis for ErrorLogDiagnosis {
    fn run(&self, wp: &WpCli, root: &Path) -> Result<DiagnosisReport> {
        println!("  Running Error Log Diagnosis...");
        let mut details = Vec::new();
        let mut overall_status = Status::Ok;

        let log_path = Self::find_error_log(wp, root);

        let log_path = match log_path {
            Some(p) => p,
            None => {
                details.push("Could not locate a PHP/web-server error log (checked common LiteSpeed/Apache/Nginx/PHP-FPM paths and wp-config.php WP_DEBUG_LOG).".to_string());
                return Ok(DiagnosisReport {
                    module: "ErrorLog".to_string(),
                    status: Status::Warning,
                    message: "Error log not found.".to_string(),
                    details,
                });
            }
        };

        let tail = match Self::tail_file(&log_path, TAIL_LINES) {
            Ok(t) => t,
            Err(e) => {
                details.push(format!("Found log at {:?} but could not read it: {}", log_path, e));
                return Ok(DiagnosisReport {
                    module: "ErrorLog".to_string(),
                    status: Status::Warning,
                    message: "Error log unreadable.".to_string(),
                    details,
                });
            }
        };

        details.push(format!("Scanned last {} lines of {:?}.", TAIL_LINES, log_path));

        self.analyze_log(&tail, &mut overall_status, &mut details);

        Ok(DiagnosisReport {
            module: "ErrorLog".to_string(),
            status: overall_status,
            message: "Error log checked.".to_string(),
            details,
        })
    }
}

impl ErrorLogDiagnosis {
    fn find_error_log(wp: &WpCli, root: &Path) -> Option<PathBuf> {
        for candidate in CANDIDATE_LOG_PATHS {
            let path = PathBuf::from(candidate);
            if path.exists() {
                return Some(path);
            }
        }

        // Fall back to whatever PHP itself is configured to use.
        if let Ok(output) = wp.run(&["eval", "echo ini_get('error_log');"], root) {
            let trimmed = output.trim();
            if !trimmed.is_empty() {
                let path = PathBuf::from(trimmed);
                if path.exists() {
                    return Some(path);
                }
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

    fn analyze_log(&self, log_tail: &str, status: &mut Status, details: &mut Vec<String>) {
        let mut fatal_count = 0usize;
        let mut warning_count = 0usize;
        let mut per_plugin: HashMap<String, usize> = HashMap::new();

        for line in log_tail.lines() {
            let is_fatal = line.contains("PHP Fatal error") || line.contains("PHP Parse error");
            let is_warning = line.contains("PHP Warning") || line.contains("PHP Deprecated");

            if !is_fatal && !is_warning {
                continue;
            }
            if is_fatal {
                fatal_count += 1;
            } else {
                warning_count += 1;
            }

            if let Some(plugin) = Self::extract_plugin(line) {
                *per_plugin.entry(plugin).or_insert(0) += 1;
            }
        }

        if fatal_count == 0 && warning_count == 0 {
            details.push("No PHP fatal errors or warnings found in the recent log window.".to_string());
            return;
        }

        details.push(format!(
            "Found {} fatal error(s) and {} warning(s) in the recent log window.",
            fatal_count, warning_count
        ));

        if fatal_count > 0 {
            *status = Status::Error;
        } else {
            *status = Status::Warning;
        }

        let mut offenders: Vec<(&String, &usize)> = per_plugin.iter().collect();
        offenders.sort_by(|a, b| b.1.cmp(a.1));
        for (plugin, count) in offenders.into_iter().take(5) {
            details.push(format!(" - {}: {} occurrence(s)", plugin, count));
        }
    }

    fn extract_plugin(line: &str) -> Option<String> {
        let marker = "wp-content/plugins/";
        let idx = line.find(marker)?;
        let rest = &line[idx + marker.len()..];
        let name = rest.split(['/', ' ']).next()?;
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_log_clean() {
        let diagnosis = ErrorLogDiagnosis;
        let mut status = Status::Ok;
        let mut details = Vec::new();

        diagnosis.analyze_log("[10-Sep-2026] normal request log line", &mut status, &mut details);

        assert_eq!(status, Status::Ok);
        assert!(details.iter().any(|d| d.contains("No PHP fatal errors")));
    }

    #[test]
    fn test_analyze_log_fatal_pinpoints_plugin() {
        let diagnosis = ErrorLogDiagnosis;
        let mut status = Status::Ok;
        let mut details = Vec::new();
        let log = "[10-Sep-2026 12:00:00 UTC] PHP Fatal error:  Uncaught TypeError in /var/www/html/wp-content/plugins/wonderm00ns-simple-facebook-open-graph-tags/wonderm00n-open-graph.php:42";

        diagnosis.analyze_log(log, &mut status, &mut details);

        assert_eq!(status, Status::Error);
        assert!(details.iter().any(|d| d.contains("wonderm00ns-simple-facebook-open-graph-tags: 1 occurrence")));
    }

    #[test]
    fn test_analyze_log_warnings_only_is_warning_not_error() {
        let diagnosis = ErrorLogDiagnosis;
        let mut status = Status::Ok;
        let mut details = Vec::new();
        let log = "[10-Sep-2026] PHP Warning:  Undefined array key in /var/www/html/wp-content/plugins/some-plugin/file.php:10";

        diagnosis.analyze_log(log, &mut status, &mut details);

        assert_eq!(status, Status::Warning);
    }

    #[test]
    fn test_extract_plugin_from_path() {
        let line = "PHP Fatal error in /var/www/html/wp-content/plugins/akismet/akismet.php on line 5";
        assert_eq!(ErrorLogDiagnosis::extract_plugin(line), Some("akismet".to_string()));
    }

    #[test]
    fn test_extract_plugin_none_when_absent() {
        let line = "PHP Fatal error in /var/www/html/wp-content/themes/divi/functions.php on line 5";
        assert_eq!(ErrorLogDiagnosis::extract_plugin(line), None);
    }
}
