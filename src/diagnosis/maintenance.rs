use crate::diagnosis::{Diagnosis, DiagnosisReport, Status};
use crate::wp::WpCli;
use std::path::Path;
use anyhow::Result;

pub struct MaintenanceDiagnosis;

impl Diagnosis for MaintenanceDiagnosis {
    fn run(&self, wp: &WpCli, root: &Path) -> Result<DiagnosisReport> {
        println!("  Running Maintenance Diagnosis...");
        let mut details = Vec::new();
        let mut overall_status = Status::Ok;

        // 1. Post Revisions
        println!("    > Checking post revisions...");
        // Count revisions: `wp post list --post_type=revision --format=count`
        match wp.run(&["post", "list", "--post_type=revision", "--format=count"], root) {
            Ok(count_str) => {
                self.analyze_revisions(&count_str, &mut overall_status, &mut details);
            }
            Err(_) => details.push("Could not count post revisions.".to_string()),
        }

        // 2. Expired Transients
        println!("    > Checking expired transients...");
        // `wp transient delete --expired --dry-run` or just assume status
        // WP-CLI doesn't have a direct "count expired" without delete command easily exposed, 
        // but `wp transient delete --expired` returns "Deleted X transients".
        // We shouldn't delete without asking.
        // Let's just check if there are many.
        // `wp option list --search="*_transient_timeout_*" --format=count` gives total transients with timeouts (approx).
        
        match wp.run(&["option", "list", "--search=*_transient_timeout_*", "--format=count"], root) {
             Ok(count_str) => {
                 let count: usize = count_str.trim().parse().unwrap_or(0);
                 details.push(format!("Found {} transient timeout records.", count));
             }
             Err(_) => {}
        }

        // 3. Check Debug Log Size
        println!("    > Checking debug.log...");
        // Check content folder usually wp-content/debug.log
        let debug_log = root.join("wp-content/debug.log");
        if debug_log.exists() {
              match std::fs::metadata(&debug_log) {
                  Ok(metadata) => {
                      self.analyze_log_size(metadata.len(), &mut overall_status, &mut details);
                  }
                  Err(_) => details.push("Could not read debug.log metadata.".to_string()),
              }
        } else {
            details.push("No debug.log found (good).".to_string());
        }

        Ok(DiagnosisReport {
            module: "Maintenance".to_string(),
            status: overall_status,
            message: "Maintenance Checked".to_string(),
            details,
        })
    }
}

impl MaintenanceDiagnosis {
    fn analyze_revisions(&self, count_str: &str, status: &mut Status, details: &mut Vec<String>) {
        let count: usize = count_str.trim().parse().unwrap_or(0);
        details.push(format!("Found {} post revisions.", count));
        
        if count > 1000 {
            *status = Status::Warning;
            details.push("Warning: High number of post revisions (> 1000). Consider cleaning them.".to_string());
        }
    }

    fn analyze_log_size(&self, size_bytes: u64, status: &mut Status, details: &mut Vec<String>) {
        let size_mb = size_bytes as f64 / 1024.0 / 1024.0;
        details.push(format!("debug.log found: {:.2} MB", size_mb));
        
        if size_mb > 50.0 {
            *status = Status::Warning;
            details.push("Warning: debug.log is very large (> 50MB).".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_revisions() {
        let diagnosis = MaintenanceDiagnosis;
        let mut status = Status::Ok;
        let mut details = Vec::new();
        
        diagnosis.analyze_revisions("1500", &mut status, &mut details);
        
        assert_eq!(status, Status::Warning);
        assert!(details.iter().any(|d| d.contains("Found 1500 post revisions")));
    }

    #[test]
    fn test_analyze_log_size() {
        let diagnosis = MaintenanceDiagnosis;
        let mut status = Status::Ok;
        let mut details = Vec::new();
        
        // 60 MB
        diagnosis.analyze_log_size(60 * 1024 * 1024, &mut status, &mut details);
        
        assert_eq!(status, Status::Warning);
        assert!(details.iter().any(|d| d.contains("debug.log found: 60.00 MB")));
    }
}
