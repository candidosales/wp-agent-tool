use crate::diagnosis::{Diagnosis, DiagnosisReport, Status};
use crate::wp::WpCli;
use std::path::Path;
use anyhow::Result;

pub struct DatabaseDiagnosis;

impl Diagnosis for DatabaseDiagnosis {
    fn run(&self, wp: &WpCli, root: &Path) -> Result<DiagnosisReport> {
        println!("  Running Database Diagnosis...");
        let mut details = Vec::new();
        let mut overall_status = Status::Ok;

        // 1. wp db check
        println!("    > Checking database integrity...");
        match wp.run(&["db", "check"], root) {
            Ok(output) => {
                details.push(format!("Check: {}", output.trim()));
            },
            Err(e) => {
                overall_status = Status::Error;
                details.push(format!("Check Error: {}", e));
            }
        }

        // 2. wp db size
        println!("    > Checking database size...");
        match wp.run(&["db", "size", "--human-readable"], root) {
            Ok(output) => {
                 details.push(format!("Size: {}", output.trim()));
            },
            Err(e) => {
                details.push(format!("Size check failed: {}", e));
            }
        }

        // 3. Optimization suggestion (or run)
        // For logic, maybe we suggest check valid logic or just ask?
        // For now, let's just reporting.
        // If we want to optimize, we should probably do it if requested or ask user.
        // but the prompt says "Optimize / Vaccuum", which sounds like an action.
        // I will make it interactive if not okay, but diagnosis usually just reports.
        // However, the user request says "run a diagnosis... Optimize / Vaccuum".
        // Maybe I should add an "Optimization" phase?
        // Or do it here interactively?
        // I'll stick to Analysis first, maybe the tool supports "--fix" later.
        // Or I can just check if optimization is needed? Hard to tell without running it.
        // I'll add a separate step for optimization in main flow, or ask user here.
        
        let report = DiagnosisReport {
            module: "Database".to_string(),
            status: overall_status,
            message: "Database checked.".to_string(),
            details,
        };
        
        Ok(report)
    }
}
