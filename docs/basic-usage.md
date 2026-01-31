# Basic Usage Guide

## Quick Start

Navigate to your WordPress project directory and run:

```bash
wp-agent
```

WP Agent will automatically:

1. Check if `wp-cli` is installed (and offer to install it if missing)
2. Locate your WordPress root directory
3. Run comprehensive diagnostics
4. Display a summary report

## Command Line Options

```bash
# Display help
wp-agent --help

# Show version
wp-agent --version
```

## Understanding the Report

WP Agent runs several diagnostic modules and displays results in a color-coded table:

- 🟢 **OK**: Everything looks good
- 🟡 **WARNING**: Potential issues that should be reviewed
- 🔴 **ERROR**: Critical issues that need immediate attention

### Diagnostic Modules

#### Database

- Runs integrity checks (`wp db check`)
- Analyzes database size
- Checks for optimization opportunities

#### Plugins

- Identifies plugins with available updates
- Lists inactive plugins
- Checks for known security issues

#### System

- Verifies PHP version compatibility
- Checks disk usage (especially `/tmp`)
- Monitors system resources

#### Network

- Verifies external connectivity
- Tests site reachability
- Checks DNS resolution

#### Security

- Verifies WordPress core checksums
- Checks if debug mode is enabled
- Audits admin users
- Reviews file permissions

#### Performance

- Analyzes autoloaded options size
- Checks cron events
- Verifies object cache status
- Reviews database query performance

#### Maintenance

- Counts post revisions
- Identifies expired transients
- Checks debug log size
- Reviews update status

## Working with WP-CLI

WP Agent automatically handles WP-CLI installation and configuration:

### Local Installation

If WP-CLI is not found in your system PATH, WP Agent will:

1. Offer to download `wp-cli.phar` to your project
2. Use the local copy for all operations
3. Store it in your project root

### Root Execution

When running as root (common on VPS environments), WP Agent will:

1. Attempt to detect the site owner
2. Switch to that user for WP-CLI commands
3. Fall back to `--allow-root` if needed

## Example Workflow

```bash
# Navigate to your WordPress site
cd /var/www/html/mysite

# Run diagnostics
wp-agent

# Review the output
# Address any WARNING or ERROR items
# Re-run to verify fixes
wp-agent
```

## Troubleshooting

### WP-CLI Not Found

If WP Agent can't find WP-CLI:

- Accept the prompt to install it locally, or
- Install WP-CLI globally: https://wp-cli.org/

### WordPress Root Not Detected

If WP Agent can't find `wp-config.php`:

- Make sure you're in or near your WordPress directory
- Check that `wp-config.php` exists and is readable

### Permission Issues

If you encounter permission errors:

- Ensure you have read access to WordPress files
- On servers, you may need to run as the web server user
- WP Agent will handle root execution automatically

## Next Steps

- Review the [Features Documentation](./features.md)
- Check the [Installation Guide](./installation.md)
- Learn about [Contributing](../CONTRIBUTING.md)
