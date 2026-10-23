use crate::diagnosis::{Diagnosis, DiagnosisReport, Status};
use crate::wp::WpCli;
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::Result;

// The Linux release is a static musl binary, so /etc/ld.so.preload never loads into
// this process: everything read here via std::fs is the kernel's view. Comparing it
// with glibc tools (ps, ls) exposes what a preload rootkit hides from them.

const PRELOAD_PATH: &str = "/etc/ld.so.preload";
const HIDDEN_FILE_DIRS: &[&str] = &["/etc", "/lib", "/usr/lib", "/usr/lib64", "/bin", "/usr/bin", "/usr/local/bin", "/tmp", "/var/tmp", "/dev/shm"];
const SUSPICIOUS_EXE_PREFIXES: &[&str] = &["/tmp/", "/var/tmp/", "/dev/shm/"];
const UPLOAD_PHP_EXTENSIONS: &[&str] = &["php", "phtml", "phar", "php5", "php7", "pht"];
const MAX_LISTED: usize = 10;

pub struct CompromiseDiagnosis;

impl Diagnosis for CompromiseDiagnosis {
    fn run(&self, _wp: &WpCli, root: &Path) -> Result<DiagnosisReport> {
        eprintln!("  Running Compromise Diagnosis...");
        let mut details = Vec::new();
        let mut findings = 0;

        // Linux-only checks; on macOS (dev machines) only the uploads check applies.
        if Path::new("/proc/self").exists() {
            eprintln!("    > Checking {}...", PRELOAD_PATH);
            findings += Self::check_preload(&mut details);

            eprintln!("    > Comparing /proc with ps for hidden processes...");
            findings += Self::check_hidden_processes(&mut details);

            eprintln!("    > Comparing directory listings with ls for hidden files...");
            findings += Self::check_hidden_files(&mut details);

            eprintln!("    > Checking process executables...");
            findings += Self::check_process_exes(&mut details);
        } else {
            details.push("Not Linux: skipped preload, hidden-process, hidden-file and executable checks.".to_string());
        }

        eprintln!("    > Looking for PHP files in uploads...");
        findings += Self::check_upload_php(root, &mut details);

        let (status, message) = if findings > 0 {
            (Status::Error, format!("{} indicator(s) of compromise. Treat the server as compromised: rebuild, don't clean.", findings))
        } else {
            (Status::Ok, "No indicators of compromise found".to_string())
        };

        Ok(DiagnosisReport {
            module: "Compromise".to_string(),
            status,
            message,
            details,
        })
    }
}

impl CompromiseDiagnosis {
    fn check_preload(details: &mut Vec<String>) -> usize {
        let Ok(contents) = fs::read_to_string(PRELOAD_PATH) else { return 0 };
        let libs: Vec<&str> = contents
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();
        if libs.is_empty() {
            return 0;
        }
        details.push(format!(
            "ERROR: {} preloads into every process: {}. Legitimate servers almost never use it; this is the classic userland-rootkit hook.",
            PRELOAD_PATH,
            libs.join(", ")
        ));
        1
    }

    fn proc_pids() -> BTreeSet<u32> {
        fs::read_dir("/proc")
            .map(|rd| rd.flatten().filter_map(|e| e.file_name().to_str()?.parse().ok()).collect())
            .unwrap_or_default()
    }

    fn check_hidden_processes(details: &mut Vec<String>) -> usize {
        // Read /proc before and after ps: only PIDs alive the whole time count, so
        // processes starting or exiting during the comparison aren't false positives.
        let before = Self::proc_pids();
        let output = match Command::new("ps").args(["-e", "-o", "pid="]).output() {
            Ok(o) if o.status.success() => o,
            _ => {
                details.push("ps not available: skipped hidden-process check.".to_string());
                return 0;
            }
        };
        let after = Self::proc_pids();
        let seen_by_ps: HashSet<u32> = String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .filter_map(|p| p.parse().ok())
            .collect();

        let hidden: Vec<String> = before
            .intersection(&after)
            .filter(|pid| !seen_by_ps.contains(pid) && **pid != std::process::id())
            .map(|pid| format!("{} ({})", pid, Self::describe_pid(*pid)))
            .collect();
        if hidden.is_empty() {
            return 0;
        }
        details.push(format!("ERROR: {} process(es) exist in /proc but are hidden from ps:", hidden.len()));
        details.extend(hidden.iter().take(MAX_LISTED).map(|h| format!("   {}", h)));
        1
    }

    fn describe_pid(pid: u32) -> String {
        let comm = fs::read_to_string(format!("/proc/{}/comm", pid)).unwrap_or_default();
        let exe = fs::read_link(format!("/proc/{}/exe", pid))
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "exe unreadable".to_string());
        format!("{}, {}", comm.trim(), exe)
    }

    fn check_hidden_files(details: &mut Vec<String>) -> usize {
        let mut hidden = Vec::new();
        let mut scanned = HashSet::new();
        for dir in HIDDEN_FILE_DIRS {
            // /lib -> /usr/lib on merged-/usr systems: scan each real dir once.
            let Ok(real) = fs::canonicalize(dir) else { continue };
            if !scanned.insert(real.clone()) {
                continue;
            }
            let Ok(output) = Command::new("ls").arg("-A1").arg(&real).output() else {
                details.push("ls not available: skipped hidden-file check.".to_string());
                return 0;
            };
            let seen_by_ls: HashSet<String> = String::from_utf8_lossy(&output.stdout).lines().map(String::from).collect();
            let Ok(entries) = fs::read_dir(&real) else { continue };
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                // ls prints a name containing '\n' as several lines.
                let listed = seen_by_ls.contains(&name) || name.split('\n').all(|p| seen_by_ls.contains(p));
                // Re-check existence so files deleted after ls ran aren't reported.
                if !listed && entry.path().symlink_metadata().is_ok() {
                    hidden.push(real.join(&name).display().to_string());
                }
            }
        }
        if hidden.is_empty() {
            return 0;
        }
        details.push(format!("ERROR: {} file(s) exist on disk but are hidden from ls:", hidden.len()));
        details.extend(hidden.iter().take(MAX_LISTED).map(|h| format!("   {}", h)));
        1
    }

    fn check_process_exes(details: &mut Vec<String>) -> usize {
        let mut suspicious = Vec::new();
        let mut dockerds = Vec::new();
        let mut unreadable = 0;
        for pid in Self::proc_pids() {
            let exe = match fs::read_link(format!("/proc/{}/exe", pid)) {
                Ok(p) => p.display().to_string(),
                Err(_) => {
                    unreadable += 1;
                    continue;
                }
            };
            // A deleted binary whose path exists again was replaced by a package upgrade
            // (process not restarted yet); self-deleting malware leaves nothing behind.
            let deleted_gone = exe
                .strip_suffix(" (deleted)")
                .is_some_and(|p| !Path::new(p).exists());
            if deleted_gone || SUSPICIOUS_EXE_PREFIXES.iter().any(|p| exe.starts_with(p)) {
                suspicious.push(format!("{} ({})", pid, exe));
            }
            if exe.trim_end_matches(" (deleted)").rsplit('/').next() == Some("dockerd") {
                dockerds.push(format!("{} ({})", pid, exe));
            }
        }

        let mut findings = 0;
        if !suspicious.is_empty() {
            details.push(format!("ERROR: {} process(es) run from a temp dir or a deleted binary:", suspicious.len()));
            details.extend(suspicious.iter().take(MAX_LISTED).map(|s| format!("   {}", s)));
            findings += 1;
        }
        if dockerds.len() > 1 {
            details.push(format!("ERROR: {} dockerd processes; a second daemon is a known way to hide cryptominers/proxyware:", dockerds.len()));
            details.extend(dockerds.iter().take(MAX_LISTED).map(|d| format!("   {}", d)));
            findings += 1;
        }
        // Kernel threads have no exe either, so only mention this when not root.
        if unreadable > 0 && unsafe { libc::geteuid() } != 0 {
            details.push("Not root: executables of other users' processes were not checked.".to_string());
        }
        findings
    }

    fn check_upload_php(root: &Path, details: &mut Vec<String>) -> usize {
        let uploads = root.join("wp-content/uploads");
        let mut found = Vec::new();
        Self::walk_php(&uploads, &mut found);
        if found.is_empty() {
            return 0;
        }
        details.push(format!("ERROR: {} PHP file(s) in wp-content/uploads, the most common webshell location:", found.len()));
        details.extend(found.iter().take(MAX_LISTED).map(|f| format!("   {}", f.display())));
        1
    }

    fn walk_php(dir: &Path, found: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_dir() {
                Self::walk_php(&path, found);
            } else if ft.is_file() && Self::is_php(&path) && !Self::is_silence_stub(&path) {
                found.push(path);
            }
        }
    }

    fn is_php(path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| UPLOAD_PHP_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
    }

    // Plugins drop `<?php // Silence is golden.` index.php files to block directory listing.
    fn is_silence_stub(path: &Path) -> bool {
        fs::metadata(path).is_ok_and(|m| m.len() < 200)
            && fs::read_to_string(path).is_ok_and(|c| c.to_ascii_lowercase().contains("silence is golden"))
    }
}
