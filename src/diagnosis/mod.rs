use std::path::Path;
use crate::wp::WpCli;
use anyhow::Result;

#[derive(Debug, PartialEq)]
pub struct DiagnosisReport {
    pub module: String,
    pub status: Status,
    pub message: String,
    pub details: Vec<String>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Status {
    Ok,
    Warning,
    Error,
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
