use std::process::Command;
use std::path::{Path, PathBuf};
use which::which;
use console::style;
use std::fs;
use std::io::copy;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::MetadataExt;

pub struct WpCli {
    executable_path: PathBuf,
}

impl WpCli {
    pub fn new() -> Self {
        // Default to "wp" in path, or specific logic to find it later
        WpCli {
            executable_path: PathBuf::from("wp"),
        }
    }

    pub fn is_installed(&self) -> bool {
        if self.executable_path == PathBuf::from("wp") {
            which("wp").is_ok()
        } else {
            self.executable_path.exists()
        }
    }

    pub fn check_and_install(&mut self) -> anyhow::Result<()> {
        if self.is_installed() {
            println!("{} WP-CLI is installed.", style("✔").green());
            return Ok(());
        }

        println!("{} WP-CLI is NOT installed globally.", style("✘").red());
        
        // Check if local phar exists from previous run
        let local_phar = std::env::current_dir()?.join("wp-cli.phar");
        if local_phar.exists() {
             println!("Found local wp-cli.phar.");
             self.executable_path = local_phar;
             return Ok(());
        }

        let confirm = dialoguer::Confirm::new()
            .with_prompt("Do you want to install WP-CLI locally (in current dir)?")
            .interact()?;

        if confirm {
            let path = self.install()?;
            self.executable_path = path;
        } else {
             return Err(anyhow::anyhow!("WP-CLI is required to proceed."));
        }

        Ok(())
    }

    fn install(&self) -> anyhow::Result<PathBuf> {
        println!("Downloading WP-CLI...");
        let target_url = "https://raw.githubusercontent.com/wp-cli/builds/gh-pages/phar/wp-cli.phar";
        let response = reqwest::blocking::get(target_url)?;
        
        let current_dir = std::env::current_dir()?;
        let wp_path = current_dir.join("wp-cli.phar");
        
        let mut dest = fs::File::create(&wp_path)?;
        let content = response.bytes()?;
        copy(&mut content.as_ref(), &mut dest)?;

        println!("Setting permissions...");
        let mut perms = fs::metadata(&wp_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&wp_path, perms)?;

        println!("{} WP-CLI installed successfully at {:?}.", style("✔").green(), wp_path);
        
        Ok(wp_path)
    }

    pub fn find_root(&self) -> anyhow::Result<PathBuf> {
        // Start from current dir and look up for wp-config.php
        let mut current_dir = std::env::current_dir()?;
        loop {
            if current_dir.join("wp-config.php").exists() {
                return Ok(current_dir);
            }
            if !current_dir.pop() {
                 break;
            }
        }

        // Fallback: Check /var/www/html
        let default_path = PathBuf::from("/var/www/html");
        if default_path.join("wp-config.php").exists() {
             println!("{} WordPress root found at default location: /var/www/html", style("✔").green());
             return Ok(default_path);
        }
        
        // If not found, ask user
        println!("{} Could not find WordPress root (wp-config.php not found).", style("?").yellow());
        let path_str: String = dialoguer::Input::new()
            .with_prompt("Please enter the path to the WordPress root directory")
            .interact_text()?;
            
        let path = PathBuf::from(&path_str);
        if path.join("wp-config.php").exists() {
            Ok(path)
        } else {
            Err(anyhow::anyhow!("Invalid WordPress path provided."))
        }
    }

    pub fn run(&self, args: &[&str], cwd: &Path) -> anyhow::Result<String> {
        let mut cmd = self.executable_path.to_string_lossy().to_string();
        let mut final_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

        // Check for root execution using libc
        let is_root = unsafe { libc::geteuid() == 0 };

        if is_root {
            let config_path = cwd.join("wp-config.php");
            let target_path = if config_path.exists() { config_path.as_path() } else { cwd };

            if let Ok(metadata) = std::fs::metadata(target_path) {
                let file_uid = metadata.uid();
                if file_uid == 0 {
                    final_args.push("--allow-root".to_string());
                } else {
                    // Construct sudo command: sudo -u #<uid> -- <existing_cmd> <args>
                    let uid_arg = format!("#{}", file_uid);
                    let mut new_args = vec![
                        "-u".to_string(),
                        uid_arg,
                        "--".to_string(),
                        cmd,
                    ];
                    new_args.append(&mut final_args);
                    
                    cmd = "sudo".to_string();
                    final_args = new_args;
                }
            }
        }

        let output = Command::new(&cmd)
            .args(&final_args)
            .current_dir(cwd)
            .output()?;

        if !output.status.success() {
             let stderr = String::from_utf8_lossy(&output.stderr);
             // handle edge case where wp-cli outputs errors to stdout sometimes or vice versa
             return Err(anyhow::anyhow!("WP-CLI failed: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
