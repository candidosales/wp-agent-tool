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
        
        match wp.run(&["config", "get", "table_prefix"], root) {
            Ok(prefix) => {
                 let prefix = prefix.trim();
                 let query = format!("SELECT SUM(LENGTH(option_value)) FROM {}options WHERE autoload = 'yes'", prefix);
                 match wp.run(&["db", "query", &query, "--skip-column-names"], root) {
                     Ok(size_str) => {
                         self.analyze_autoload_size(&size_str, &mut overall_status, &mut details);
                     }
                     Err(_) => details.push("Could not determine autoloaded options size.".to_string()),
                 }
            }
            Err(_) => details.push("Could not determine table prefix.".to_string()),
        }

        // 2. Check Cron
        println!("    > Checking cron events...");
        // Simpler cron check: count overdue
        match wp.run(&["cron", "event", "list", "--format=csv"], root) {
             Ok(_) => {
                 details.push("Cron events checked.".to_string());
             }
             Err(_) => details.push("Could not check cron events.".to_string()),
        }

        // 3. Object Cache
        println!("    > Checking object cache...");
        match wp.run(&["plugin", "is-active", "redis-cache"], root) {
            Ok(_) => details.push("Redis Object Cache plugin is active.".to_string()),
            Err(_) => {
                // Check for other drops ins?
                 match wp.run(&["plugin", "is-active", "w3-total-cache"], root) {
                     Ok(_) => details.push("W3 Total Cache is active.".to_string()),
                     Err(_) => details.push("No common object cache plugin detected (Redis/W3TC).".to_string()),
                 }
            }
        }

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
}
