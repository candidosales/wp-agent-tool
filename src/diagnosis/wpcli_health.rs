use crate::diagnosis::{Diagnosis, DiagnosisReport, Status};
use crate::wp::WpCli;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;
use anyhow::Result;

/// wp-cli releases older than this predate PHP 8 compatibility fixes in
/// several bundled commands (e.g. cache-command's argument-order bug in
/// DocParser), so they can fatal outright once the site's PHP is upgraded.
const MIN_HEALTHY_MAJOR_MINOR: (u32, u32) = (2, 5);

pub struct WpCliHealthDiagnosis;

impl Diagnosis for WpCliHealthDiagnosis {
    fn run(&self, wp: &WpCli, root: &Path) -> Result<DiagnosisReport> {
        println!("  Running WP-CLI Health Diagnosis...");
        let mut details = Vec::new();
        let mut status = Status::Ok;

        // 1. Can wp-cli actually boot WordPress, not just run its own commands?
        match wp.run(&["option", "get", "siteurl"], root) {
            Ok(_) => details.push("wp-cli booted WordPress successfully.".to_string()),
            Err(e) => {
                status = Self::escalate(status, Status::Error);
                Self::classify_boot_failure(&e.to_string(), &mut details);
            }
        }

        // 2 & 3. wp-cli's own version, and the PHP version it runs on vs. the
        // web server's PHP version — a mismatch here is exactly what silently
        // broke wp-cli while the live site kept working fine in a real incident.
        match Self::cli_info(wp, root) {
            Some((cli_version, php_version)) => {
                details.push(format!("wp-cli version: {}", cli_version));
                if let Some((maj, min)) = Self::parse_major_minor(&cli_version) {
                    if (maj, min) < MIN_HEALTHY_MAJOR_MINOR {
                        status = Self::escalate(status, Status::Warning);
                        details.push(format!(
                            "Warning: wp-cli {} is old — versions before {}.{} carry known PHP 8 \
                             incompatibility bugs in bundled commands. Download a current wp-cli.phar.",
                            cli_version, MIN_HEALTHY_MAJOR_MINOR.0, MIN_HEALTHY_MAJOR_MINOR.1
                        ));
                    }
                }

                details.push(format!("wp-cli runs on PHP {}.", php_version));
                let web_php_versions = Self::detect_web_php_versions();
                if web_php_versions.is_empty() {
                    details.push(
                        "Could not auto-detect the web server's PHP version for comparison.".to_string(),
                    );
                } else {
                    details.push(format!(
                        "Web server PHP version(s) detected: {}.",
                        web_php_versions.join(", ")
                    ));
                    let matches_any = web_php_versions
                        .iter()
                        .any(|v| Self::same_major_minor(v, &php_version));
                    if !matches_any {
                        status = Self::escalate(status, Status::Warning);
                        details.push(format!(
                            "Warning: wp-cli runs on a different PHP version ({}) than the web server \
                             ({}). A plugin/core-library bug tied to one PHP version can silently break \
                             wp-cli while the live site keeps working fine, or vice versa.",
                            php_version,
                            web_php_versions.join("/")
                        ));
                    }
                }
            }
            None => {
                details.push(
                    "Could not determine wp-cli's version or PHP runtime (wp-cli itself may be broken \
                     beyond running any command)."
                        .to_string(),
                );
            }
        }

        Ok(DiagnosisReport {
            module: "WpCliHealth".to_string(),
            status,
            message: "WP-CLI health checked.".to_string(),
            details,
        })
    }
}

impl WpCliHealthDiagnosis {
    fn escalate(current: Status, candidate: Status) -> Status {
        match (current, candidate) {
            (Status::Error, _) | (_, Status::Error) => Status::Error,
            (Status::Warning, _) | (_, Status::Warning) => Status::Warning,
            _ => Status::Ok,
        }
    }

    fn classify_boot_failure(err: &str, details: &mut Vec<String>) {
        let truncated = Self::truncate(err);
        if err.contains("must be compatible with") {
            details.push(format!(
                "wp-cli failed to boot WordPress with a PHP class-declaration incompatibility error:\n{}",
                truncated
            ));
            details.push(
                "This pattern (\"Declaration of X::method() must be compatible with Y::method()\") is \
                 often a PHP-version-specific engine bug in a bundled/plugin library that only manifests \
                 on the PHP version wp-cli runs under, not a real incompatibility in the code — check the \
                 PHP version cross-check below."
                    .to_string(),
            );
        } else {
            details.push(format!("wp-cli failed to boot WordPress:\n{}", truncated));
        }
        details.push(
            "Diagnoses below will fall back to raw DB queries where possible; some checks may be \
             skipped or incomplete."
                .to_string(),
        );
    }

    fn truncate(s: &str) -> String {
        const MAX_CHARS: usize = 500;
        let s = s.trim();
        let truncated: String = s.chars().take(MAX_CHARS).collect();
        if s.chars().count() > MAX_CHARS {
            format!("{}... (truncated)", truncated)
        } else {
            truncated
        }
    }

    fn parse_major_minor(v: &str) -> Option<(u32, u32)> {
        let mut parts = v.trim().split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        Some((major, minor))
    }

    fn same_major_minor(a: &str, b: &str) -> bool {
        Self::parse_major_minor(a) == Self::parse_major_minor(b)
    }

    /// Returns (wp-cli version, PHP version) from `wp cli info`, which works
    /// even when other wp-cli commands fatal, since it doesn't boot WordPress.
    fn cli_info(wp: &WpCli, root: &Path) -> Option<(String, String)> {
        let output = wp.run(&["cli", "info"], root).ok()?;

        let mut cli_version = None;
        let mut php_version = None;
        for line in output.lines() {
            if let Some(v) = line.strip_prefix("WP-CLI version:") {
                cli_version = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("PHP version:") {
                php_version = Some(v.trim().to_string());
            }
        }

        Some((cli_version?, php_version?))
    }

    /// Best-effort detection of the PHP version(s) actually serving web
    /// traffic, by finding running php-fpm/lsphp worker binaries and asking
    /// each for its version directly.
    fn detect_web_php_versions() -> Vec<String> {
        let output = match Command::new("ps").args(["-eo", "args"]).output() {
            Ok(o) => o,
            Err(_) => return Vec::new(),
        };
        let text = String::from_utf8_lossy(&output.stdout);

        let mut binaries: HashSet<String> = HashSet::new();
        for line in text.lines() {
            let first_token = line.trim().split_whitespace().next().unwrap_or("");
            if !first_token.starts_with('/') {
                continue;
            }
            if first_token.contains("php-fpm") || first_token.contains("lsphp") {
                binaries.insert(first_token.to_string());
            }
        }

        let mut versions = Vec::new();
        for bin in binaries {
            if let Ok(out) = Command::new(&bin).arg("-v").output() {
                let text = String::from_utf8_lossy(&out.stdout);
                if let Some(v) = Self::extract_php_version(&text) {
                    if !versions.contains(&v) {
                        versions.push(v);
                    }
                }
            }
        }
        versions
    }

    fn extract_php_version(text: &str) -> Option<String> {
        // e.g. first line: "PHP 8.1.34 (litespeed) (built: ...)" or "PHP 7.4.3-4ubuntu2.29 (cli) ..."
        let first_line = text.lines().next()?;
        let rest = first_line.strip_prefix("PHP ")?;
        Some(rest.split_whitespace().next()?.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_major_minor() {
        assert_eq!(WpCliHealthDiagnosis::parse_major_minor("2.12.0"), Some((2, 12)));
        assert_eq!(WpCliHealthDiagnosis::parse_major_minor("2.2.0"), Some((2, 2)));
        assert_eq!(WpCliHealthDiagnosis::parse_major_minor("bogus"), None);
    }

    #[test]
    fn test_same_major_minor() {
        assert!(WpCliHealthDiagnosis::same_major_minor("8.1.34", "8.1.2"));
        assert!(!WpCliHealthDiagnosis::same_major_minor("8.1.34", "7.4.3"));
    }

    #[test]
    fn test_extract_php_version() {
        let text = "PHP 8.1.34 (litespeed) (built: Dec 24 2025 08:20:24)\nCopyright (c) ...";
        assert_eq!(
            WpCliHealthDiagnosis::extract_php_version(text),
            Some("8.1.34".to_string())
        );

        let text_cli = "PHP 7.4.3-4ubuntu2.29 (cli) (built: Mar 25 2025 18:57:03) ( NTS )";
        assert_eq!(
            WpCliHealthDiagnosis::extract_php_version(text_cli),
            Some("7.4.3-4ubuntu2.29".to_string())
        );
    }

    #[test]
    fn test_classify_boot_failure_declaration_incompatibility() {
        let mut details = Vec::new();
        let err = "WP-CLI failed: PHP Fatal error:  Declaration of Foo::fromArray(array $array): Foo must be compatible with Bar::fromArray(array $array): Bar";
        WpCliHealthDiagnosis::classify_boot_failure(err, &mut details);

        assert!(details.iter().any(|d| d.contains("class-declaration incompatibility")));
        assert!(details.iter().any(|d| d.contains("PHP-version-specific engine bug")));
    }

    #[test]
    fn test_classify_boot_failure_generic() {
        let mut details = Vec::new();
        let err = "WP-CLI failed: PHP Fatal error: Call to undefined function foo()";
        WpCliHealthDiagnosis::classify_boot_failure(err, &mut details);

        assert!(details.iter().any(|d| d.contains("Call to undefined function")));
        assert!(!details.iter().any(|d| d.contains("class-declaration incompatibility")));
    }

    #[test]
    fn test_truncate_long_error() {
        let long = "x".repeat(1000);
        let truncated = WpCliHealthDiagnosis::truncate(&long);
        assert!(truncated.ends_with("... (truncated)"));
        assert!(truncated.len() < long.len());
    }

    #[test]
    fn test_escalate_error_wins() {
        assert_eq!(
            WpCliHealthDiagnosis::escalate(Status::Error, Status::Warning),
            Status::Error
        );
        assert_eq!(
            WpCliHealthDiagnosis::escalate(Status::Warning, Status::Ok),
            Status::Warning
        );
        assert_eq!(WpCliHealthDiagnosis::escalate(Status::Ok, Status::Ok), Status::Ok);
    }
}
