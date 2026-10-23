use crate::diagnosis::{Diagnosis, DiagnosisReport, Status};
use crate::wp::WpCli;
use std::path::Path;
use anyhow::Result;

/// wp-cli releases older than this predate PHP 8 compatibility fixes in
/// several bundled commands (e.g. cache-command's argument-order bug in
/// DocParser), so they can fatal outright once the site's PHP is upgraded.
const MIN_HEALTHY_MAJOR_MINOR: (u32, u32) = (2, 5);
const MAX_ERROR_CHARS: usize = 500;

pub struct WpCliHealthDiagnosis;

impl Diagnosis for WpCliHealthDiagnosis {
    fn run(&self, wp: &WpCli, root: &Path) -> Result<DiagnosisReport> {
        eprintln!("  Running WP-CLI Health Diagnosis...");
        let mut details = Vec::new();
        let mut status = Status::Ok;

        // `wp cli info` runs without WordPress; `option get` must boot it.
        eprintln!("    > Checking wp-cli can boot WordPress...");
        match wp.run(&["option", "get", "siteurl"], root) {
            Ok(_) => details.push("wp-cli booted WordPress successfully.".to_string()),
            Err(e) => {
                status = Status::Error;
                Self::classify_boot_failure(&e.to_string(), &mut details);
            }
        }

        eprintln!("    > Checking wp-cli version...");
        let version = wp.cli_info(root).ok().map(|i| i.wp_cli_version.clone()).filter(|v| !v.is_empty());
        match version {
            Some(v) => {
                details.push(format!("wp-cli version: {}", v));
                if Self::major_minor(&v).is_some_and(|mm| mm < MIN_HEALTHY_MAJOR_MINOR) {
                    if status == Status::Ok {
                        status = Status::Warning;
                    }
                    details.push(format!(
                        "Warning: wp-cli {} is older than {}.{} and carries known PHP 8 incompatibility bugs in bundled commands. Download a current wp-cli.phar.",
                        v, MIN_HEALTHY_MAJOR_MINOR.0, MIN_HEALTHY_MAJOR_MINOR.1
                    ));
                }
            }
            None => details.push("Could not read wp-cli's version: wp-cli itself may be broken.".to_string()),
        }

        let message = match status {
            Status::Ok => "wp-cli boots WordPress",
            Status::Warning => "wp-cli is outdated",
            Status::Error => "wp-cli cannot boot WordPress",
        };
        Ok(DiagnosisReport { module: "WpCliHealth".to_string(), status, message: message.to_string(), details })
    }
}

impl WpCliHealthDiagnosis {
    fn classify_boot_failure(err: &str, details: &mut Vec<String>) {
        let err = err.trim();
        let mut shown: String = err.chars().take(MAX_ERROR_CHARS).collect();
        if err.chars().count() > MAX_ERROR_CHARS {
            shown.push_str("... (truncated)");
        }
        details.push(format!("ERROR: wp-cli failed to boot WordPress: {}", shown));
        if err.contains("must be compatible with") {
            details.push(
                "\"Declaration of X::method() must be compatible with Y::method()\" is often a PHP-version-specific engine bug that only hits the PHP wp-cli runs under. Check the Patches module for a wp-cli vs. web server PHP mismatch."
                    .to_string(),
            );
        }
        details.push("Modules below fall back to raw DB queries where possible; some checks may be skipped.".to_string());
    }

    fn major_minor(v: &str) -> Option<(u32, u32)> {
        let mut parts = v.trim().split('.');
        Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
    }
}
