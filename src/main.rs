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

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    
    if args.debug > 0 {
        println!("Debug mode enabled");
    }

    println!("{}", style("WP Agent starting...").bold().cyan());
    
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
    println!("{} WordPress root found at: {:?}", style("✔").green(), root);

    // 3. Run Diagnoses
    println!("\nRunning diagnoses...");
    
    let modules: Vec<Box<dyn Diagnosis>> = vec![
        Box::new(DatabaseDiagnosis),
        Box::new(PluginDiagnosis),
        Box::new(SystemDiagnosis),
        Box::new(NetworkDiagnosis),
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
    report::display(&reports);
    
    Ok(())
}
