mod cli;
mod wp;
mod diagnosis;
mod report;

use clap::Parser;
use cli::Cli;
use wp::WpCli;
use console::style;
use diagnosis::{Diagnosis, DiagnosisReport};
use diagnosis::database::DatabaseDiagnosis;
use diagnosis::plugins::PluginDiagnosis;
use diagnosis::system::SystemDiagnosis;
use diagnosis::network::NetworkDiagnosis;
use diagnosis::security::SecurityDiagnosis;
use diagnosis::performance::PerformanceDiagnosis;
use diagnosis::maintenance::MaintenanceDiagnosis;
use diagnosis::errorlog::ErrorLogDiagnosis;
use diagnosis::scanner::ScannerDiagnosis;
use diagnosis::patches::PatchesDiagnosis;
use diagnosis::compromise::CompromiseDiagnosis;
use diagnosis::exposure::ExposureDiagnosis;
use diagnosis::mail::MailDiagnosis;

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    
    if args.debug > 0 {
        eprintln!("Debug mode enabled");
    }

    eprintln!("{}", style("WP Agent starting...").bold().cyan());
    
    let mut wp = WpCli::new();
    
    // 1. Check/Install WP-CLI
    if let Err(e) = wp.check_and_install() {
        eprintln!("{} {}", style("Error:").red(), e);
        std::process::exit(1);
    }
    
    // 2. Find WP Root
    let root = match wp.find_root() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{} {}", style("Error:").red(), e);
            std::process::exit(1);
        }
    };
    eprintln!("{} WordPress root found at: {:?}", style("✔").green(), root);

    // 3. Run Diagnoses
    eprintln!("\nRunning diagnoses...");
    
    let modules: Vec<Box<dyn Diagnosis>> = vec![
        Box::new(DatabaseDiagnosis),
        Box::new(PluginDiagnosis),
        Box::new(SystemDiagnosis),
        Box::new(PatchesDiagnosis),
        Box::new(NetworkDiagnosis),
        Box::new(SecurityDiagnosis),
        Box::new(CompromiseDiagnosis),
        Box::new(ExposureDiagnosis),
        Box::new(MailDiagnosis),
        Box::new(PerformanceDiagnosis),
        Box::new(MaintenanceDiagnosis),
        Box::new(ErrorLogDiagnosis),
        Box::new(ScannerDiagnosis),
    ];
    
    let mut reports = Vec::new();
    
    for module in modules {
        match module.run(&wp, &root) {
            Ok(report) => reports.push(report),
            Err(e) => {
                eprintln!("{} Diagnosis module failed: {}", style("Error:").red(), e);
                // Create a generic error report for failure
                reports.push(DiagnosisReport {
                    module: "Unknown".to_string(),
                    status: diagnosis::Status::Error,
                    message: format!("Module execution failed: {}", e),
                    details: vec![],
                });
            }
        }
    }
    
    // 4. Report
    if args.json {
        println!("{}", serde_json::to_string_pretty(&reports)?);
    } else {
        report::display(&reports);
    }

    // 1 = could not run, 2 = ran and found errors, so cron/CI can alert on findings.
    if reports.iter().any(|r| r.status == diagnosis::Status::Error) {
        std::process::exit(2);
    }
    Ok(())
}
