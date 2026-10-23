use crate::diagnosis::{Diagnosis, DiagnosisReport, Status};
use crate::wp::WpCli;
use std::path::Path;
use std::process::Command;
use anyhow::Result;

// Active plugins whose slug contains one of these send wp_mail() over their own SMTP/API.
const MAIL_PLUGIN_HINTS: &[&str] = &["smtp", "sendgrid", "mailgun", "postmark", "sparkpost", "wp-ses", "brevo", "sendinblue"];
const SENDMAIL_PATHS: &[&str] = &["/usr/sbin/sendmail", "/usr/lib/sendmail"];

pub struct MailDiagnosis;

impl Diagnosis for MailDiagnosis {
    fn run(&self, wp: &WpCli, root: &Path) -> Result<DiagnosisReport> {
        eprintln!("  Running Mail Diagnosis...");
        let mut details = Vec::new();
        let mut status = Status::Ok;

        eprintln!("    > Looking for an active SMTP plugin...");
        let plugin = match wp.active_plugins(root) {
            Ok(plugins) => plugins.into_iter().find(|n| MAIL_PLUGIN_HINTS.iter().any(|h| n.contains(h))),
            Err(_) => {
                details.push("Could not list active plugins (wp-cli and raw DB both failed): could not check for an SMTP plugin.".to_string());
                None
            }
        };

        eprintln!("    > Checking the local MTA...");
        if let Some(p) = &plugin {
            details.push(format!("WordPress sends mail through the {} plugin.", p));
        } else if let Some(relay) = Self::postconf("relayhost") {
            if relay.is_empty() {
                Self::raise(&mut status, Status::Warning);
                details.push("Warning: no SMTP plugin and Postfix delivers directly from this server's IP. Gmail and Outlook reject that mail (550 5.7.26) unless the domain's SPF/DKIM/DMARC cover this IP, so password resets never arrive. Set a Postfix relayhost or install an SMTP plugin.".to_string());
            } else {
                details.push(format!("Postfix relays mail through {}.", relay));
                Self::check_sasl(&mut status, &mut details);
            }
        } else if SENDMAIL_PATHS.iter().any(|p| Path::new(p).exists()) {
            details.push("A non-Postfix MTA provides sendmail: relay settings not checked.".to_string());
        } else {
            Self::raise(&mut status, Status::Warning);
            details.push("Warning: no SMTP plugin and no sendmail on this server: wp_mail() fails, so password resets and notifications are never sent.".to_string());
        }

        eprintln!("    > Checking the mail queue...");
        Self::check_queue(&mut status, &mut details);

        let message = match status {
            Status::Ok => "Outgoing mail is configured",
            Status::Warning => "Outgoing mail likely not delivered",
            Status::Error => "Outgoing mail is broken",
        };
        Ok(DiagnosisReport { module: "Mail".to_string(), status, message: message.to_string(), details })
    }
}

impl MailDiagnosis {
    fn raise(status: &mut Status, to: Status) {
        let rank = |s: &Status| match s { Status::Ok => 0, Status::Warning => 1, Status::Error => 2 };
        if rank(&to) > rank(status) {
            *status = to;
        }
    }

    /// Value of a Postfix parameter; None when Postfix isn't installed.
    fn postconf(param: &str) -> Option<String> {
        let out = Command::new("postconf").args(["-h", param]).output().ok().filter(|o| o.status.success())?;
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    fn check_sasl(status: &mut Status, details: &mut Vec<String>) {
        if Self::postconf("smtp_sasl_auth_enable").as_deref() != Some("yes") {
            return;
        }
        // "hash:/etc/postfix/sasl_passwd" is only read from its postmap output, sasl_passwd.db.
        let maps = Self::postconf("smtp_sasl_password_maps").unwrap_or_default();
        let missing: Vec<String> = maps
            .split([',', ' '])
            .filter(|m| !m.is_empty())
            .filter_map(|m| {
                let (kind, path) = m.split_once(':')?;
                let file = match kind {
                    "hash" | "btree" => format!("{}.db", path),
                    "lmdb" => format!("{}.lmdb", path),
                    "texthash" => path.to_string(),
                    _ => return None,
                };
                (!Path::new(&file).exists()).then_some(file)
            })
            .collect();
        if !missing.is_empty() {
            Self::raise(status, Status::Error);
            details.push(format!(
                "ERROR: SMTP authentication is on but the password map {} doesn't exist: the relay refuses every message. Create the credentials file and run postmap on it.",
                missing.join(", ")
            ));
        }
    }

    fn check_queue(status: &mut Status, details: &mut Vec<String>) {
        // One JSON object per queued message (Postfix >= 3.1); fails when Postfix isn't running.
        let Some(out) = Command::new("postqueue").arg("-j").output().ok().filter(|o| o.status.success()) else { return };
        let queued: Vec<serde_json::Value> = String::from_utf8_lossy(&out.stdout).lines().filter_map(|l| serde_json::from_str(l).ok()).collect();
        if queued.is_empty() {
            details.push("Mail queue is empty.".to_string());
            return;
        }
        let reason = queued
            .iter()
            .flat_map(|m| m.get("recipients").and_then(|r| r.as_array()).cloned().unwrap_or_default())
            .find_map(|r| r.get("delay_reason")?.as_str().map(String::from))
            .unwrap_or_else(|| "not attempted yet".to_string());
        Self::raise(status, Status::Warning);
        details.push(format!("Warning: {} message(s) stuck in the Postfix queue. Delivery error: {}", queued.len(), reason));
    }
}
