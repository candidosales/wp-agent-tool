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

- **Autoloaded options**: Analyzes size of autoloaded data
- **Cron events**: Reviews scheduled tasks and their frequency
- **Object cache**: Checks if object caching is enabled
- **Database queries**: Identifies slow or problematic queries
- **Transients**: Reports on transient usage and cleanup

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
