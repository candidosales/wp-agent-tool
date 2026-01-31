use crate::diagnosis::{Diagnosis, DiagnosisReport, Status};
use crate::wp::WpCli;
use std::path::Path;
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Plugin {
    name: String,
    status: String,
    update: String, // "available" or "none"
    version: String,
}

pub struct PluginDiagnosis;

impl Diagnosis for PluginDiagnosis {
    fn run(&self, wp: &WpCli, root: &Path) -> Result<DiagnosisReport> {
        println!("  Running Plugin Diagnosis...");
        
        println!("    > Fetching plugin list...");
        let output = wp.run(&["plugin", "list", "--format=json"], root)?;
        
        self.analyze_plugins(&output)
    }
}

impl PluginDiagnosis {
    fn analyze_plugins(&self, json_output: &str) -> Result<DiagnosisReport> {
        let mut details = Vec::new();
        let mut overall_status = Status::Ok;

        // Parse JSON
        let plugins: Vec<Plugin> = serde_json::from_str(json_output)?;
        
        // Check for updates
        let updates_available: Vec<&Plugin> = plugins.iter().filter(|p| p.update == "available").collect();
        if !updates_available.is_empty() {
             overall_status = Status::Warning;
             details.push(format!("{} plugins have updates available:", updates_available.len()));
             for p in updates_available {
                 details.push(format!(" - {} ({})", p.name, p.version));
             }
        } else {
             details.push("All plugins are up to date.".to_string());
        }

        // Check for inactive plugins
        let inactive: Vec<&Plugin> = plugins.iter().filter(|p| p.status == "inactive").collect();
        if !inactive.is_empty() {
             details.push(format!("{} inactive plugins found (consider removing checks):", inactive.len()));
             for p in inactive {
                  details.push(format!(" - {}", p.name));
             }
        }
        
        Ok(DiagnosisReport {
            module: "Plugins".to_string(),
            status: overall_status,
            message: format!("Analyzed {} plugins.", plugins.len()),
            details,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_plugins_updates() -> Result<()> {
        let json = r#"[
            {"name": "akismet", "status": "active", "update": "available", "version": "5.0"},
            {"name": "hello", "status": "inactive", "update": "none", "version": "1.7.2"}
        ]"#;
        
        let diagnosis = PluginDiagnosis;
        let report = diagnosis.analyze_plugins(json)?;
        
        assert_eq!(report.module, "Plugins");
        matches!(report.status, Status::Warning);
        assert!(report.details.iter().any(|d| d.contains("1 plugins have updates available")));
        assert!(report.details.iter().any(|d| d.contains("inactive plugins found")));
        Ok(())
    }

    #[test]
    fn test_analyze_plugins_all_ok() -> Result<()> {
        let json = r#"[
            {"name": "akismet", "status": "active", "update": "none", "version": "5.0"}
        ]"#;
        
        let diagnosis = PluginDiagnosis;
        let report = diagnosis.analyze_plugins(json)?;
        
        matches!(report.status, Status::Ok);
        assert!(report.details.iter().any(|d| d.contains("All plugins are up to date")));
        Ok(())
    }
}
