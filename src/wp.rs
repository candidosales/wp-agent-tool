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
            eprintln!("{} WP-CLI is installed.", style("✔").green());
            return Ok(());
        }

        eprintln!("{} WP-CLI is NOT installed globally.", style("✘").red());
        
        // Check if local phar exists from previous run
        let local_phar = std::env::current_dir()?.join("wp-cli.phar");
        if local_phar.exists() {
             eprintln!("Found local wp-cli.phar.");
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
        eprintln!("Downloading WP-CLI...");
        let target_url = "https://raw.githubusercontent.com/wp-cli/builds/gh-pages/phar/wp-cli.phar";
        let response = reqwest::blocking::get(target_url)?;
        
        let current_dir = std::env::current_dir()?;
        let wp_path = current_dir.join("wp-cli.phar");
        
        let mut dest = fs::File::create(&wp_path)?;
        let content = response.bytes()?;
        copy(&mut content.as_ref(), &mut dest)?;

        eprintln!("Setting permissions...");
        let mut perms = fs::metadata(&wp_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&wp_path, perms)?;

        eprintln!("{} WP-CLI installed successfully at {:?}.", style("✔").green(), wp_path);
        
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
             eprintln!("{} WordPress root found at default location: /var/www/html", style("✔").green());
             return Ok(default_path);
        }
        
        // If not found, ask user
        eprintln!("{} Could not find WordPress root (wp-config.php not found).", style("?").yellow());
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

    /// Parses DB_NAME/DB_USER/DB_PASSWORD/DB_HOST straight out of wp-config.php.
    /// Used as a fallback when wp-cli itself is broken (e.g. a fatal error in
    /// WordPress core/a plugin makes every `wp` invocation fail, as seen when
    /// a core bug crashed wp-cli entirely on a live site).
    fn db_config(&self, root: &Path) -> anyhow::Result<(String, String, String, String)> {
        let config_path = root.join("wp-config.php");
        let contents = fs::read_to_string(&config_path)?;

        let extract = |key: &str| -> anyhow::Result<String> {
            let needle = format!("define('{}'", key);
            let needle_dq = format!("define(\"{}\"", key);
            // Stock wp-config.php writes `define( 'DB_NAME', ... )`, with spaces.
            let line = contents
                .lines()
                .find(|l| {
                    let l: String = l.chars().filter(|c| !c.is_whitespace()).collect();
                    l.contains(&needle) || l.contains(&needle_dq)
                })
                .ok_or_else(|| anyhow::anyhow!("{} not found in wp-config.php", key))?;

            let mut parts = line.splitn(3, ['\'', '"']);
            parts.next(); // define( prefix up to first quote
            parts.next(); // the key itself
            let rest = parts.next().unwrap_or("");
            let mut rest_parts = rest.splitn(3, ['\'', '"']);
            rest_parts.next(); // separator/comma up to opening quote of value
            let value = rest_parts.next().unwrap_or("").to_string();
            Ok(value)
        };

        let name = extract("DB_NAME")?;
        let user = extract("DB_USER")?;
        let pass = extract("DB_PASSWORD").unwrap_or_default();
        let host = extract("DB_HOST").unwrap_or_else(|_| "localhost".to_string());

        Ok((name, user, pass, host))
    }

    /// Read-only SQL via `wp db query`, falling back to the `mysql` client when
    /// wp-cli itself is broken. Writes must go through wp-cli: the fallback skips
    /// WordPress's own cache invalidation.
    pub fn query(&self, root: &Path, sql: &str) -> anyhow::Result<String> {
        let wp_err = match self.run(&["db", "query", sql, "--skip-column-names"], root) {
            Ok(out) => return Ok(out.trim().to_string()),
            Err(e) => e,
        };
        self.raw_query(root, sql)
            .map_err(|raw_err| anyhow::anyhow!("{}; raw DB fallback also failed: {}", wp_err, raw_err))
    }

    /// `$table_prefix`, read from wp-config.php when `wp config get` fails.
    pub fn table_prefix(&self, root: &Path) -> anyhow::Result<String> {
        if let Ok(prefix) = self.run(&["config", "get", "table_prefix"], root) {
            return Ok(prefix.trim().to_string());
        }
        let contents = fs::read_to_string(root.join("wp-config.php"))?;
        contents
            .lines()
            .find_map(|l| {
                let rest = l.trim().strip_prefix("$table_prefix")?;
                let mut parts = rest.split(['\'', '"']);
                parts.next(); // ` = ` up to the opening quote
                parts.next().map(String::from)
            })
            .ok_or_else(|| anyhow::anyhow!("$table_prefix not found in wp-config.php"))
    }

    /// Slugs of active plugins, read from the `active_plugins` option when wp-cli
    /// can't boot WordPress.
    pub fn active_plugins(&self, root: &Path) -> anyhow::Result<Vec<String>> {
        let wp_err = match self.run(&["plugin", "list", "--status=active", "--field=name"], root) {
            Ok(out) => return Ok(out.lines().map(str::trim).filter(|l| !l.is_empty()).map(String::from).collect()),
            Err(e) => e,
        };
        let prefix = self.table_prefix(root)?;
        let sql = format!("SELECT option_value FROM {}options WHERE option_name = 'active_plugins'", prefix);
        let serialized = self
            .raw_query(root, &sql)
            .map_err(|raw_err| anyhow::anyhow!("{}; raw DB fallback also failed: {}", wp_err, raw_err))?;
        // a:2:{i:0;s:19:"akismet/akismet.php";i:1;s:9:"hello.php";} -> akismet, hello
        Ok(serialized
            .split('"')
            .skip(1)
            .step_by(2)
            .map(|file| file.split('/').next().unwrap_or(file).trim_end_matches(".php").to_string())
            .collect())
    }

    fn raw_query(&self, root: &Path, query: &str) -> anyhow::Result<String> {
        let (name, user, pass, host) = self.db_config(root)?;

        // MYSQL_PWD, not -p<pass>: argv is readable by every local user via ps.
        let output = Command::new("mysql")
            .args(["-h", &host, "-u", &user, "-N", "-e", query, &name])
            .env("MYSQL_PWD", &pass)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("raw mysql query failed: {}", stderr.trim()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
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
             return Err(anyhow::anyhow!("WP-CLI failed: {}", stderr.trim()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
