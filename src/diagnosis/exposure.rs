use crate::diagnosis::{Diagnosis, DiagnosisReport, Status};
use crate::wp::WpCli;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::Result;

// The web server serves wp-content/uploads as static files, so anything left there
// is downloadable by anyone who guesses or finds the URL.
const ARCHIVE_EXTENSIONS: &[&str] = &["zip", "gz", "tgz", "tar", "bak", "7z", "rar"];
const DUMP_EXTENSIONS: &[&str] = &["sql"];
// Export plugins write CSV/XML dumps of posts and users next to their archives.
const EXPORT_DIRS: &[&str] = &["wpallexport", "wp-all-export", "backwpup", "updraft", "ai1wm-backups"];
const RECENT_DAYS: u32 = 90;
const MAX_LISTED: usize = 10;

pub struct ExposureDiagnosis;

impl Diagnosis for ExposureDiagnosis {
    fn run(&self, wp: &WpCli, root: &Path) -> Result<DiagnosisReport> {
        eprintln!("  Running Exposure Diagnosis...");
        let mut details = Vec::new();
        let mut status = Status::Ok;

        eprintln!("    > Looking for backups and exports in uploads...");
        Self::check_uploads(root, &mut status, &mut details);

        eprintln!("    > Checking recently registered administrators and editors...");
        Self::check_recent_users(wp, root, &mut status, &mut details);

        let message = match status {
            Status::Ok => "No public backups or new privileged users",
            Status::Warning => "Review exposed files and new privileged users",
            Status::Error => "Database dump publicly downloadable",
        };
        Ok(DiagnosisReport { module: "Exposure".to_string(), status, message: message.to_string(), details })
    }
}

impl ExposureDiagnosis {
    fn raise(status: &mut Status, to: Status) {
        let rank = |s: &Status| match s { Status::Ok => 0, Status::Warning => 1, Status::Error => 2 };
        if rank(&to) > rank(status) {
            *status = to;
        }
    }

    fn check_uploads(root: &Path, status: &mut Status, details: &mut Vec<String>) {
        let uploads = root.join("wp-content/uploads");
        let mut dumps = Vec::new();
        let mut archives = Vec::new();
        Self::walk(&uploads, false, &mut dumps, &mut archives);
        let rel = |p: &PathBuf| p.strip_prefix(root).unwrap_or(p).display().to_string();

        if !dumps.is_empty() {
            Self::raise(status, Status::Error);
            details.push(format!("ERROR: {} database dump(s) in uploads, downloadable with every user's password hash and email:", dumps.len()));
            details.extend(dumps.iter().take(MAX_LISTED).map(|p| format!("   {}", rel(p))));
        }
        if !archives.is_empty() {
            Self::raise(status, Status::Warning);
            details.push(format!(
                "Warning: {} archive/export file(s) in uploads are publicly downloadable (.htaccess deny rules don't apply on OpenLiteSpeed or nginx). Delete them or move them outside the web root:",
                archives.len()
            ));
            details.extend(archives.iter().take(MAX_LISTED).map(|p| format!("   {}", rel(p))));
        }
    }

    fn walk(dir: &Path, in_export_dir: bool, dumps: &mut Vec<PathBuf>, archives: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(ft) = entry.file_type() else { continue };
            let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            if ft.is_dir() {
                Self::walk(&path, in_export_dir || EXPORT_DIRS.contains(&name.as_str()), dumps, archives);
                continue;
            }
            if !ft.is_file() {
                continue;
            }
            let ext = name.rsplit('.').next().unwrap_or("");
            if DUMP_EXTENSIONS.contains(&ext) || name.ends_with(".sql.gz") {
                dumps.push(path);
            } else if ARCHIVE_EXTENSIONS.contains(&ext) || (in_export_dir && matches!(ext, "csv" | "xml" | "json")) {
                archives.push(path);
            }
        }
    }

    fn check_recent_users(wp: &WpCli, root: &Path, status: &mut Status, details: &mut Vec<String>) {
        let args = ["user", "list", "--role__in=administrator,editor", "--fields=user_login,user_email,user_registered,roles", "--format=json"];
        let users: Vec<serde_json::Value> = match wp.run(&args, root).ok().and_then(|o| serde_json::from_str(&o).ok()) {
            Some(u) => u,
            None => {
                details.push("wp user list failed: skipped recent-administrator check.".to_string());
                return;
            }
        };
        let Some(cutoff) = Self::days_ago(RECENT_DAYS) else {
            details.push("date unavailable: skipped recent-administrator check.".to_string());
            return;
        };
        let field = |u: &serde_json::Value, k: &str| u.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
        // user_registered is "YYYY-MM-DD HH:MM:SS" (UTC), so string order is date order.
        let recent: Vec<String> = users
            .iter()
            .filter(|u| field(u, "user_registered") >= cutoff)
            .map(|u| format!("   {} <{}> {}, registered {}", field(u, "user_login"), field(u, "user_email"), field(u, "roles"), field(u, "user_registered")))
            .collect();
        if recent.is_empty() {
            details.push(format!("No administrators or editors registered in the last {} days.", RECENT_DAYS));
            return;
        }
        Self::raise(status, Status::Warning);
        details.push(format!(
            "Warning: {} administrator/editor account(s) registered in the last {} days. Attackers add these for persistence; confirm each one is known:",
            recent.len(),
            RECENT_DAYS
        ));
        details.extend(recent.into_iter().take(MAX_LISTED));
    }

    /// UTC date `days` ago as "YYYY-MM-DD" (GNU date, then BSD date).
    fn days_ago(days: u32) -> Option<String> {
        let gnu = vec!["-u".to_string(), "-d".to_string(), format!("{} days ago", days), "+%Y-%m-%d".to_string()];
        let bsd = vec!["-u".to_string(), format!("-v-{}d", days), "+%Y-%m-%d".to_string()];
        [gnu, bsd].into_iter().find_map(|args| {
            let out = Command::new("date").args(&args).output().ok().filter(|o| o.status.success())?;
            Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
        })
    }
}
