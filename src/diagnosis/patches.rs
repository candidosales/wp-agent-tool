use crate::diagnosis::{Diagnosis, DiagnosisReport, Status};
use crate::wp::WpCli;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use anyhow::Result;

const PHP_ACTIVE_URL: &str = "https://www.php.net/releases/active.php";
// Used when php.net is unreachable. Update on each PHP security release.
const FALLBACK_LATEST: &[(&str, &str)] = &[("8.2", "8.2.35"), ("8.3", "8.3.36"), ("8.4", "8.4.27"), ("8.5", "8.5.12")];
const FALLBACK_DATE: &str = "2026-10-21";
// Ubuntu LTS end of standard support (YYYY-MM).
const UBUNTU_LTS_EOL: &[(&str, &str)] = &[
    ("14.04", "2019-04"), ("16.04", "2021-04"), ("18.04", "2023-05"), ("20.04", "2025-05"),
    ("22.04", "2027-06"), ("24.04", "2029-05"), ("26.04", "2031-05"),
];
const PHP_SERVER_PREFIXES: &[&str] = &["lsphp", "php-fpm", "php-cgi"];

pub struct PatchesDiagnosis;

impl Diagnosis for PatchesDiagnosis {
    fn run(&self, wp: &WpCli, root: &Path) -> Result<DiagnosisReport> {
        println!("  Running Patches Diagnosis...");
        let mut details = Vec::new();
        let mut status = Status::Ok;

        println!("    > Fetching supported PHP branches from php.net...");
        let latest = match Self::fetch_latest() {
            Ok(l) => l,
            Err(e) => {
                details.push(format!("php.net unreachable ({}); using built-in release list from {}.", e, FALLBACK_DATE));
                FALLBACK_LATEST.iter().map(|(b, v)| (b.to_string(), v.to_string())).collect()
            }
        };

        println!("    > Checking the PHP the web server runs...");
        let served = Self::served_php();
        if served.is_empty() {
            details.push("No PHP worker running right now (LiteSpeed starts them on demand): only wp-cli's PHP was checked.".to_string());
        }
        for (exe, version) in &served {
            Self::check_php(&format!("Web server PHP ({})", exe), version, &latest, &mut status, &mut details);
        }

        println!("    > Checking wp-cli's PHP...");
        if let Some(version) = Self::wp_cli_php(wp, root) {
            Self::check_php("wp-cli PHP", &version, &latest, &mut status, &mut details);
            let served_branches: BTreeSet<String> = served.iter().map(|(_, v)| Self::branch(v)).collect();
            if !served_branches.is_empty() && !served_branches.contains(&Self::branch(&version)) {
                Self::raise(&mut status, Status::Warning);
                details.push("Warning: wp-cli runs a different PHP branch than the web server, so wp-cli results (and plugin compatibility checks) don't reflect the live site.".to_string());
            }
        }

        println!("    > Checking OS support status...");
        Self::check_os(&mut status, &mut details);

        let message = match status {
            Status::Ok => "PHP and OS are on supported, patched releases",
            Status::Warning => "Patch status needs attention",
            Status::Error => "Unpatched or end-of-life PHP in use",
        };
        Ok(DiagnosisReport { module: "Patches".to_string(), status, message: message.to_string(), details })
    }
}

impl PatchesDiagnosis {
    fn raise(status: &mut Status, to: Status) {
        let rank = |s: &Status| match s { Status::Ok => 0, Status::Warning => 1, Status::Error => 2 };
        if rank(&to) > rank(status) {
            *status = to;
        }
    }

    /// branch ("8.3") -> latest release ("8.3.36") for every supported branch.
    fn fetch_latest() -> Result<BTreeMap<String, String>> {
        let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(10)).build()?;
        let json: serde_json::Value = serde_json::from_str(&client.get(PHP_ACTIVE_URL).send()?.error_for_status()?.text()?)?;
        let latest: BTreeMap<String, String> = json
            .as_object()
            .into_iter()
            .flat_map(|majors| majors.values())
            .filter_map(|branches| branches.as_object())
            .flat_map(|branches| branches.iter())
            .filter_map(|(b, info)| Some((b.clone(), info.get("version")?.as_str()?.to_string())))
            .collect();
        anyhow::ensure!(!latest.is_empty(), "no branches in response");
        Ok(latest)
    }

    /// Distinct PHP executables run by the web server, with `<exe> -v` output's version.
    fn served_php() -> Vec<(String, String)> {
        let mut exes = BTreeSet::new();
        for entry in fs::read_dir("/proc").into_iter().flatten().flatten() {
            if !entry.file_name().to_string_lossy().bytes().all(|b| b.is_ascii_digit()) {
                continue;
            }
            let Ok(exe) = fs::read_link(entry.path().join("exe")) else { continue };
            let name = exe.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            if PHP_SERVER_PREFIXES.iter().any(|p| name.starts_with(p)) {
                exes.insert(exe.display().to_string());
            }
        }
        exes.into_iter()
            .filter_map(|exe| {
                let out = Command::new(&exe).arg("-v").output().ok()?;
                let version = Self::parse_php_v(&String::from_utf8_lossy(&out.stdout))?;
                Some((exe, version))
            })
            .collect()
    }

    /// "PHP 8.3.36 (litespeed) (built: ...)" -> "8.3.36"
    fn parse_php_v(output: &str) -> Option<String> {
        output.lines().next()?.strip_prefix("PHP ")?.split_whitespace().next().map(String::from)
    }

    fn wp_cli_php(wp: &WpCli, root: &Path) -> Option<String> {
        let out = wp.run(&["cli", "info", "--format=json"], root).ok()?;
        let json: serde_json::Value = serde_json::from_str(&out).ok()?;
        json.get("php_version")?.as_str().map(String::from)
    }

    /// "8.3.36-1+ubuntu24.04.1" -> "8.3"
    fn branch(version: &str) -> String {
        version.split('.').take(2).collect::<Vec<_>>().join(".")
    }

    fn numeric(version: &str) -> Vec<u32> {
        version
            .split(|c: char| !c.is_ascii_digit() && c != '.')
            .next()
            .unwrap_or("")
            .split('.')
            .filter_map(|p| p.parse().ok())
            .collect()
    }

    fn check_php(label: &str, version: &str, latest: &BTreeMap<String, String>, status: &mut Status, details: &mut Vec<String>) {
        let branch = Self::branch(version);
        // Ubuntu/Debian builds backport fixes without bumping the upstream version (and keep
        // patching branches php.net dropped); Ondřej Surý's PPA (deb.sury.org) tracks upstream.
        let lower = version.to_ascii_lowercase();
        let distro_build = (lower.contains("ubuntu") || lower.contains("deb")) && !lower.contains("sury");
        let Some(newest) = latest.get(&branch) else {
            let oldest = latest.keys().next().map(String::as_str).unwrap_or("?");
            if Self::numeric(version) < Self::numeric(oldest) && distro_build {
                Self::raise(status, Status::Warning);
                details.push(format!("Warning: {} is {}: branch {} is end of life upstream; only the distro's security team still patches it. Plan a move to {} or newer.", label, version, branch, oldest));
            } else if Self::numeric(version) < Self::numeric(oldest) {
                Self::raise(status, Status::Error);
                details.push(format!("ERROR: {} is {}: branch {} is end of life and gets no security fixes. Move to {} or newer.", label, version, branch, oldest));
            } else {
                details.push(format!("{} is {} (branch {} not yet released as stable on php.net).", label, version, branch));
            }
            return;
        };
        if Self::numeric(version) >= Self::numeric(newest) {
            details.push(format!("{} is {}: latest {} release.", label, version, branch));
        } else if distro_build {
            Self::raise(status, Status::Warning);
            details.push(format!("Warning: {} is {}, a distro build (upstream latest {}). Distros backport fixes without changing the version: confirm with `apt list --upgradable` that no php update is pending.", label, version, newest));
        } else {
            Self::raise(status, Status::Error);
            details.push(format!("ERROR: {} is {}, missing security fixes. Latest {} release is {}.", label, version, branch, newest));
        }
    }

    fn check_os(status: &mut Status, details: &mut Vec<String>) {
        let Ok(os_release) = fs::read_to_string("/etc/os-release") else { return };
        let field = |key: &str| {
            os_release
                .lines()
                .find_map(|l| l.strip_prefix(&format!("{}=", key)))
                .map(|v| v.trim_matches('"').to_string())
                .unwrap_or_default()
        };
        if field("ID") != "ubuntu" {
            return;
        }
        let version = field("VERSION_ID");
        let Some((_, eol)) = UBUNTU_LTS_EOL.iter().find(|(v, _)| *v == version) else {
            if !version.ends_with(".04") || version.split('.').next().and_then(|y| y.parse::<u32>().ok()).is_some_and(|y| y % 2 == 1) {
                Self::raise(status, Status::Warning);
                details.push(format!("Warning: Ubuntu {} is an interim release with 9 months of support. Use an LTS release on servers.", version));
            }
            return;
        };
        let Some(today) = Command::new("date").arg("+%Y-%m").output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()) else { return };
        if today.as_str() > *eol {
            Self::raise(status, Status::Warning);
            details.push(format!(
                "Warning: Ubuntu {} standard support ended {}. LiteSpeed and PHP PPAs stop publishing builds for EOL releases, so PHP can't be patched until the OS is upgraded.",
                version, eol
            ));
        } else {
            details.push(format!("Ubuntu {} is supported until {}.", version, eol));
        }
    }
}
