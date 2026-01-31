# Features

WP Agent provides comprehensive WordPress diagnostics and maintenance checks.

## Core Features

### 🔧 WP-CLI Management

- **Automatic Detection**: Checks for `wp-cli` in your system PATH
- **Local Installation**: Offers to install `wp-cli.phar` locally if not found
- **Smart Execution**: Handles root execution by switching users or using `--allow-root`

### 📍 Intelligent Root Detection

- **Automatic Discovery**: Searches current and parent directories for `wp-config.php`
- **Manual Override**: Prompts for path if WordPress root cannot be found
- **Validation**: Verifies the directory contains a valid WordPress installation

### 🔐 Root Execution Support

- **User Detection**: Identifies the WordPress site owner
- **Safe Switching**: Executes WP-CLI commands as the appropriate user
- **Fallback Mode**: Uses `--allow-root` when user switching isn't possible

## Diagnostic Modules

### 💾 Database

- **Integrity Checks**: Runs `wp db check` to verify database health
- **Size Analysis**: Reports total database size
- **Table Statistics**: Shows number of tables and largest tables
- **Optimization**: Identifies tables that could benefit from optimization

### 🔌 Plugins

- **Update Detection**: Lists plugins with available updates
- **Inactive Plugins**: Identifies installed but inactive plugins
- **Version Information**: Shows current and available versions
- **Security Alerts**: Flags plugins with known vulnerabilities (when available)

### 💻 System

- **PHP Version**: Checks PHP version and WordPress compatibility
- **Disk Usage**: Monitors disk space, especially `/tmp` directory
- **Memory Limits**: Reviews PHP memory settings
- **Server Information**: Reports OS and server configuration

### 🌐 Network

- **External Connectivity**: Tests connection to external services (e.g., Google)
- **Site Reachability**: Verifies your WordPress site is accessible
- **DNS Resolution**: Checks domain name resolution
- **API Connectivity**: Tests WordPress.org API access

### 🛡️ Security

- **Core Checksums**: Verifies WordPress core file integrity using `wp core verify-checksums`
- **Debug Mode**: Checks if `WP_DEBUG` is enabled in production
- **Admin Users**: Audits administrator accounts
- **File Permissions**: Reviews critical file and directory permissions
- **SSL/HTTPS**: Verifies SSL certificate status

### ⚡ Performance

- **Autoloaded Options**: Analyzes size of autoloaded data
- **Cron Events**: Reviews scheduled tasks and their frequency
- **Object Cache**: Checks if object caching is enabled
- **Database Queries**: Identifies slow or problematic queries
- **Transients**: Reports on transient usage and cleanup

### 🔨 Maintenance

- **Post Revisions**: Counts total post revisions
- **Expired Transients**: Identifies transients that can be cleaned up
- **Debug Log**: Checks `debug.log` file size
- **Update Status**: Shows pending WordPress, plugin, and theme updates
- **Backup Recommendations**: Suggests backup strategies

## Summary Reporting

### Color-Coded Status

- 🟢 **OK**: No issues detected
- 🟡 **WARNING**: Potential issues that should be reviewed
- 🔴 **ERROR**: Critical issues requiring immediate attention

### Detailed Output

Each module provides:

- Status indicator
- Issue count
- Detailed findings
- Recommended actions

### Example Output

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

## Planned Features

- [ ] Export reports to JSON/HTML
- [ ] Scheduled diagnostics with email notifications
- [ ] Integration with monitoring services
- [ ] Custom diagnostic modules
- [ ] Automated fix suggestions
- [ ] Historical trend analysis

## Learn More

- [Installation Guide](./installation.md)
- [Basic Usage](./basic-usage.md)
- [Contributing](../CONTRIBUTING.md)
