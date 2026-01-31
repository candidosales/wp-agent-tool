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
        let mut details = Vec::new();
        let mut overall_status = Status::Ok;

        let output = wp.run(&["plugin", "list", "--format=json"], root)?;
        // Parse JSON
        let plugins: Vec<Plugin> = serde_json::from_str(&output)?;
        
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
