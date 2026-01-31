use crate::diagnosis::{Diagnosis, DiagnosisReport, Status};
use crate::wp::WpCli;
use std::path::Path;
use anyhow::Result;
use sysinfo::Disks;

pub struct SystemDiagnosis;

impl Diagnosis for SystemDiagnosis {
    fn run(&self, wp: &WpCli, root: &Path) -> Result<DiagnosisReport> {
        println!("  Running System Diagnosis..."); // Keep println or logging?
        let mut details = Vec::new();
        let mut overall_status = Status::Ok;
        
        // ... (rest is same logic, just fixing the arg names)
        
        // 2. Tmp folder analysis
        let disks = Disks::new_with_refreshed_list();
        let mut tmp_found = false;
        
        for disk in &disks {
            if disk.mount_point() == Path::new("/tmp") || disk.mount_point() == Path::new("/") {
                // If /tmp is not separate mount, check /
                tmp_found = true;
                let total = disk.total_space();
                let available = disk.available_space();
                let usage = 100.0 - ((available as f64 / total as f64) * 100.0);
                
                details.push(format!("Disk {:?}: {:.2}% used ({}/{} bytes)", disk.mount_point(), usage, available, total));
                
                if usage > 90.0 {
                    overall_status = Status::Warning;
                    details.push("Warning: Disk usage is high (>90%).".to_string());
                }
            }
        }
        
        if !tmp_found {
             details.push("Could not explicitly identify /tmp mount point.".to_string());
        }

        // 3. PHP Version
        match wp.run(&["cli", "info", "--format=json"], root) {
            Ok(output) => {
                 #[derive(serde::Deserialize)]
                 struct WpInfo {
                     php_version: String,
                 }
                 // Parse JSON from output string
                 match serde_json::from_str::<WpInfo>(&output) {
                     Ok(info) => {
                         details.push(format!("PHP Version: {}", info.php_version));
                         if info.php_version.starts_with("7.") || info.php_version.starts_with("5.") {
                              overall_status = Status::Warning;
                              details.push("Warning: PHP version is old. Recommend upgrading to 8.0+".to_string());
                         }
                     },
                     Err(e) => {
                         details.push(format!("Failed to parse WP info: {}", e));
                     }
                 }
            },
            Err(e) => {
                details.push(format!("Could not retrieve PHP version info: {}", e));
            }
        }
        
        Ok(DiagnosisReport {
            module: "System".to_string(),
            status: overall_status,
            message: "System Checked".to_string(),
            details,
        })
    }
}
