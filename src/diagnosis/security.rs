use crate::diagnosis::{Diagnosis, DiagnosisReport, Status};
use crate::wp::WpCli;
use std::path::Path;
use anyhow::Result;

pub struct SecurityDiagnosis;

impl Diagnosis for SecurityDiagnosis {
    fn run(&self, wp: &WpCli, root: &Path) -> Result<DiagnosisReport> {
        println!("  Running Security Diagnosis...");
        let mut details = Vec::new();
        let mut overall_status = Status::Ok;

        // 1. Core Verify Checksums
        println!("    > Verifying core checksums...");
        match wp.run(&["core", "verify-checksums"], root) {
            Ok(_) => {
                details.push("Core checksums verified.".to_string());
            }
            Err(_) => {
                overall_status = Status::Warning;
                details.push("Warning: Core checksums verification failed. Core files may be modified.".to_string());
            }
        }

        // 2. Check Debug Mode
        println!("    > Checking WP_DEBUG status...");
        match wp.run(&["config", "get", "WP_DEBUG"], root) {
            Ok(output) => {
                if output.trim() == "true" || output.trim() == "1" {
                     overall_status = Status::Warning;
                     details.push("Warning: WP_DEBUG is enabled.".to_string());
                } else {
                     details.push("WP_DEBUG is disabled.".to_string());
                }
            }
            Err(_) => {
                 details.push("Could not check WP_DEBUG status.".to_string());
            }
        }

        // 3. User Audit (Admin check)
        println!("    > Checking for admin user...");
        match wp.run(&["user", "list", "--role=administrator", "--field=user_login", "--format=csv"], root) {
             Ok(output) => {
                 self.analyze_users(&output, &mut overall_status, &mut details);
             }
             Err(_) => {
                 details.push("Could not list users.".to_string());
             }
        }

        Ok(DiagnosisReport {
            module: "Security".to_string(),
            status: overall_status,
            message: "Security Checked".to_string(),
            details,
        })
    }
}

impl SecurityDiagnosis {
    fn analyze_users(&self, csv_output: &str, status: &mut Status, details: &mut Vec<String>) {
        let admins: Vec<&str> = csv_output.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && *l != "user_login")
            .collect();
        let admin_count = admins.len();
        details.push(format!("Found {} administrator(s).", admin_count));

        if admins.contains(&"admin") {
            *status = Status::Warning;
            details.push("Warning: Insecure username 'admin' exists.".to_string());
        }
        
        if admin_count > 3 {
            details.push(format!("Note: There are {} administrators.", admin_count));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_users_with_admin() {
        let diagnosis = SecurityDiagnosis;
        let mut status = Status::Ok;
        let mut details = Vec::new();
        let csv = "user_login\nadmin\nuser1";
        
        diagnosis.analyze_users(csv, &mut status, &mut details);
        
        assert_eq!(status, Status::Warning);
        assert!(details.iter().any(|d| d.contains("Insecure username 'admin' exists")));
        assert!(details.iter().any(|d| d.contains("Found 2 administrator(s)")));
    }

    #[test]
    fn test_analyze_users_safe() {
        let diagnosis = SecurityDiagnosis;
        let mut status = Status::Ok;
        let mut details = Vec::new();
        let csv = "user_login\nbob\nalice";
        
        diagnosis.analyze_users(csv, &mut status, &mut details);
        
        assert_eq!(status, Status::Ok);
        assert!(!details.iter().any(|d| d.contains("Warning")));
    }
}
