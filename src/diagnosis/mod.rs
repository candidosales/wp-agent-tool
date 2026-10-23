use std::path::Path;
use crate::wp::WpCli;
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub struct DiagnosisReport {
    pub module: String,
    pub status: Status,
    pub message: String,
    pub details: Vec<String>,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub enum Status {
    Ok,
    Warning,
    Error,
}

/// Last `max_lines` lines of a log, reading at most the final 8 MiB so huge
/// unrotated logs don't exhaust memory. Lossy: one invalid UTF-8 byte in a log
/// written by other software must not hide the rest of it.
pub fn tail_file(path: &Path, max_lines: usize) -> std::io::Result<String> {
    use std::io::{Read, Seek, SeekFrom};
    const MAX_BYTES: u64 = 8 * 1024 * 1024;
    let mut file = std::fs::File::open(path)?;
    let start = file.metadata()?.len().saturating_sub(MAX_BYTES);
    file.seek(SeekFrom::Start(start))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let text = String::from_utf8_lossy(&buf);
    let mut lines: Vec<&str> = text.lines().collect();
    if start > 0 && !lines.is_empty() {
        lines.remove(0); // partial line cut by the seek
    }
    let skip = lines.len().saturating_sub(max_lines);
    Ok(lines[skip..].join("\n"))
}

pub trait Diagnosis {
    fn run(&self, wp: &WpCli, root: &Path) -> Result<DiagnosisReport>;
}

pub mod database;
pub mod plugins;
pub mod system;
pub mod network;
pub mod security;
pub mod performance;
pub mod maintenance;
pub mod errorlog;
pub mod scanner;
pub mod patches;
pub mod compromise;
pub mod exposure;
pub mod mail;
pub mod wpcli_health;
