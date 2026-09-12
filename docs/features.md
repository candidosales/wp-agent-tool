# Features

WP Agent provides comprehensive WordPress diagnostics and maintenance checks.

## Core features

### 🔧 WP-CLI management

- **Automatic detection**: Checks for `wp-cli` in your system PATH
- **Local installation**: Offers to install `wp-cli.phar` locally if not found
- **Smart execution**: Handles root execution by switching users or using `--allow-root`

### 📍 Intelligent root detection

- **Automatic Discovery**: Searches current and parent directories for `wp-config.php`
- **Manual Override**: Prompts for path if WordPress root cannot be found
- **Validation**: Verifies the directory contains a valid WordPress installation

### 🔐 Root execution support

- **User detection**: Identifies the WordPress site owner
- **Safe switching**: Executes WP-CLI commands as the appropriate user
- **Fallback mode**: Uses `--allow-root` when user switching isn't possible

## Diagnostic modules

### 💾 Database

- **Integrity checks**: Runs `wp db check` to verify database health
- **Size analysis**: Reports total database size
- **Table statistics**: Shows number of tables and largest tables
- **Optimization**: Identifies tables that could benefit from optimization

### 🔌 Plugins

- **Update detection**: Lists plugins with available updates
- **Inactive plugins**: Identifies installed but inactive plugins
- **Version information**: Shows current and available versions
- **Security alerts**: Flags plugins with known vulnerabilities (when available)
- **Redundancy detection**: Flags overlapping active plugins in the same category (security scanners, image optimizers, caching, SEO, backups) — running more than one adds per-request overhead for no benefit

### 💻 System

- **PHP version**: Checks PHP version and WordPress compatibility
- **Disk usage**: Monitors disk space, especially `/tmp` directory
- **Memory limits**: Reviews PHP memory settings
- **Server information**: Reports OS and server configuration

### 🌐 Network

- **External connectivity**: Tests connection to external services (e.g., Google)
- **Site reachability**: Verifies your WordPress site is accessible
- **DNS resolution**: Checks domain name resolution
- **API connectivity**: Tests WordPress.org API access

### 🛡️ Security

- **Core checksums**: Verifies WordPress core file integrity using `wp core verify-checksums`
- **Debug mode**: Checks if `WP_DEBUG` is enabled in production
- **Admin users**: Audits administrator accounts
- **File permissions**: Reviews critical file and directory permissions
- **SSL/HTTPS**: Verifies SSL certificate status

### ⚡ Performance

- **Autoloaded options**: Analyzes size of autoloaded data (falls back to a raw DB query if wp-cli fails)
- **Cron events**: Checks whether `DISABLE_WP_CRON` is set and, if so, verifies a system crontab/`/etc/cron.d` entry actually hits `wp-cron.php` — not just that a cron command ran
- **Object cache**: Detects Redis, W3 Total Cache, or LiteSpeed Cache, plus an `object-cache.php` drop-in or a running Memcached process
- **PHP limits**: Checks `memory_limit` and `max_execution_time` against the number of active plugins, warning when they're too low for the site's footprint
- **Database queries**: Identifies slow or problematic queries
- **Transients**: Reports on transient usage and cleanup

### 📄 Error log

- **Fatal error / warning scan**: Tails the PHP/web-server error log (LiteSpeed, Apache, Nginx, or PHP-FPM) and reports fatal errors and warnings from the recent window
- **Offender ranking**: Attributes errors to the plugin path they came from, so the noisiest plugin surfaces first
- **Noisy-plugin flagging**: Separately calls out plugins that log a high volume of warnings with zero fatals — safe to deprioritize, but worth reporting upstream

### 🕵️ Scanner / bot traffic

- **Access log analysis**: Tails the web-server access log (LiteSpeed, Apache, or Nginx) and flags IPs with bursty 404s or hits on known-malicious paths (`wp-config.php~`, `/.env`, `/actuator/`, `/graphql`, `.zip`/`.sql`/`.bak`, etc.)
- **fail2ban awareness**: Detects whether `fail2ban` is installed and active, escalating to ERROR when scanner traffic is present but unmitigated
- **Load-vs-traffic correlation**: Compares CPU load average against the request volume in the scanned window, distinguishing traffic-driven load from a runaway process/plugin bug

### 🔨 Maintenance

- **Post revisions**: Counts total post revisions
- **Expired transients**: Identifies transients that can be cleaned up
- **Debug log**: Checks `debug.log` file size
- **Update status**: Shows pending WordPress, plugin, and theme updates
- **Backup recommendations**: Suggests backup strategies
## Summary Reporting

### Color-coded status

- 🟢 **OK**: No issues detected
- 🟡 **WARNING**: Potential issues that should be reviewed
- 🔴 **ERROR**: Critical issues requiring immediate attention

### Detailed output

Each module provides:

- Status indicator
- Issue count
- Detailed findings
- Recommended actions

### Example output

```
╭─────────────────────────────────────────────────╮
│           WP Agent Diagnostic Report            │
├─────────────┬──────────┬────────────────────────┤
│ Module      │ Status   │ Details                │
├─────────────┼──────────┼────────────────────────┤
│ Database    │ ✓ OK     │ All checks passed      │
│ Plugins     │ ⚠ WARN   │ 3 updates available    │
│ System      │ ✓ OK     │ PHP 8.2, 45% disk used │
│ Network     │ ✓ OK     │ All connections OK     │
│ Security    │ ✗ ERROR  │ Debug mode enabled     │
│ Performance │ ⚠ WARN   │ No object cache        │
│ Error Log   │ ✗ ERROR  │ 2 fatal errors found   │
│ Scanner     │ ✗ ERROR  │ Scanner IPs, no f2b    │
│ Maintenance │ ✓ OK     │ Up to date             │
╰─────────────┴──────────┴────────────────────────╯
```

## Planned features

- [ ] Export reports to JSON/HTML
- [ ] Scheduled diagnostics with email notifications
- [ ] Integration with monitoring services
- [ ] Custom diagnostic modules
- [ ] Automated fix suggestions
- [ ] Historical trend analysis

## Learn more

- [Installation guide](./installation.md)
- [Basic usage](./basic-usage.md)
- [Contributing](../CONTRIBUTING.md)
