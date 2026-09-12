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

/// (category, keywords to match against the plugin slug). Only active plugins
/// are checked; more than one match per category is flagged, since redundant
/// plugins in the same category (e.g. two security scanners, three image
/// optimizers) each add per-request overhead without added benefit.
const REDUNDANCY_CATEGORIES: &[(&str, &[&str])] = &[
    ("security scanner", &["sucuri", "wordfence", "ithemes-security", "better-wp-security", "all-in-one-wp-security", "malcare", "shield-security"]),
    ("image optimizer", &["ewww-image-optimizer", "resmushit", "smush", "shortpixel", "imagify", "optimole", "tiny-compress-images"]),
    ("caching", &["litespeed-cache", "w3-total-cache", "wp-super-cache", "wp-rocket", "wp-fastest-cache", "redis-cache", "cache-enabler"]),
    ("seo", &["wordpress-seo", "all-in-one-seo-pack", "seo-by-rank-math", "the-seo-framework"]),
    ("backup", &["updraftplus", "backwpup", "duplicator", "all-in-one-wp-migration"]),
];

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

        self.analyze_redundancy(&plugins, &mut overall_status, &mut details);

        Ok(DiagnosisReport {
            module: "Plugins".to_string(),
            status: overall_status,
            message: format!("Analyzed {} plugins.", plugins.len()),
            details,
        })
    }

    fn analyze_redundancy(&self, plugins: &[Plugin], status: &mut Status, details: &mut Vec<String>) {
        let active: Vec<&Plugin> = plugins.iter().filter(|p| p.status == "active").collect();

        for (category, keywords) in REDUNDANCY_CATEGORIES {
            let matches: Vec<&str> = active
                .iter()
                .filter(|p| keywords.iter().any(|k| p.name.contains(k)))
                .map(|p| p.name.as_str())
                .collect();

            if matches.len() > 1 {
                *status = Status::Warning;
                details.push(format!(
                    "Warning: {} active plugins overlap in the '{}' category ({}) — running more than one adds redundant per-request overhead.",
                    matches.len(),
                    category,
                    matches.join(", ")
                ));
            }
        }
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

    #[test]
    fn test_analyze_plugins_detects_redundant_security_scanners() -> Result<()> {
        let json = r#"[
            {"name": "sucuri-scanner", "status": "active", "update": "none", "version": "1.0"},
            {"name": "wordfence", "status": "active", "update": "none", "version": "7.0"}
        ]"#;

        let diagnosis = PluginDiagnosis;
        let report = diagnosis.analyze_plugins(json)?;

        assert_eq!(report.status, Status::Warning);
        assert!(report.details.iter().any(|d| d.contains("security scanner") && d.contains("sucuri-scanner") && d.contains("wordfence")));
        Ok(())
    }

    #[test]
    fn test_analyze_plugins_no_redundancy_when_single_per_category() -> Result<()> {
        let json = r#"[
            {"name": "sucuri-scanner", "status": "active", "update": "none", "version": "1.0"},
            {"name": "akismet", "status": "active", "update": "none", "version": "5.0"}
        ]"#;

        let diagnosis = PluginDiagnosis;
        let report = diagnosis.analyze_plugins(json)?;

        assert_eq!(report.status, Status::Ok);
        Ok(())
    }
}
