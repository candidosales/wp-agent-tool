use crate::diagnosis::{DiagnosisReport, Status};
use console::style;

pub fn display(reports: &[DiagnosisReport]) {
    println!("\n{}", style("=== WP Agent Diagnosis Report ===").bold().underlined());
    
    // Summary Table
    println!("\n{:<20} | {:<10} | {}", "Module", "Status", "Message");
    println!("{:-<20}-|-{:-<10}-|-{:-<40}", "", "", "");
    
    for report in reports {
        let status_str = match report.status {
            Status::Ok => style("OK").green(),
            Status::Warning => style("WARNING").yellow(),
            Status::Error => style("ERROR").red(),
        };
        
        println!("{:<20} | {:<10} | {}", report.module, status_str, report.message);
    }
    
    // Details
    println!("\n{}", style("=== Details ===").bold());
    let mut clean_bill_of_health = true;
    
    for report in reports {
        if let Status::Ok = report.status {
             continue; // Skip details for OK unless verbose? For now skip.
        }
        clean_bill_of_health = false;
        
        let header_style = match report.status {
            Status::Warning => style(&report.module).yellow().bold(),
            Status::Error => style(&report.module).red().bold(),
            _ => style(&report.module).white(),
        };
        
        println!("\nSTATUS for {}:", header_style);
        for detail in &report.details {
            println!(" - {}", detail);
        }
    }
    
    if clean_bill_of_health {
        println!("\n{}", style("Great! No issues found.").green().bold());
    } else {
        println!("\n{}", style("Please review the warnings/errors above.").yellow());
    }
}
