use crate::diagnosis::{Diagnosis, DiagnosisReport, Status};
use crate::wp::WpCli;
use std::path::Path;
use anyhow::Result;

pub struct NetworkDiagnosis;

impl Diagnosis for NetworkDiagnosis {
    fn run(&self, wp: &WpCli, root: &Path) -> Result<DiagnosisReport> {
        println!("  Running Network Diagnosis...");
        let mut details = Vec::new();
        let mut overall_status = Status::Ok;

        // 1. Check external connectivity (google.com)
        // Using reqwest
        match reqwest::blocking::get("https://www.google.com") {
            Ok(_) => details.push("External connectivity (Google): OK".to_string()),
            Err(e) => {
                overall_status = Status::Warning;
                details.push(format!("External connectivity failed: {}", e));
            }
        }

        // 2. Check WordPress Site Reachability
        // Get site URL
        match wp.run(&["option", "get", "home"], root) {
            Ok(url) => {
                let url = url.trim();
                details.push(format!("Site URL: {}", url));
                match reqwest::blocking::get(url) {
                    Ok(resp) => {
                        let status = resp.status();
                        if status.is_success() {
                            details.push(format!("Site reachable: OK ({})", status));
                        } else {
                            overall_status = Status::Warning;
                            details.push(format!("Site returned status: {}", status));
                        }
                    },
                    Err(e) => {
                        overall_status = Status::Error; // Critical if site is down
                        details.push(format!("Site reachable failed: {}", e));
                    }
                }
            },
            Err(e) => {
                 details.push(format!("Could not get site URL: {}", e));
            }
        }

        Ok(DiagnosisReport {
            module: "Network".to_string(),
            status: overall_status,
            message: "Network Checked".to_string(),
            details,
        })
    }
}
