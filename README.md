# WP Agent

<p align="center">
 <picture>
  <source media="(prefers-color-scheme: dark)" srcset="./docs/logo.png">
  <img alt="Text changing depending on mode. Light: 'So light!' Dark: 'So dark!'" src="./docs/logo.png">
</picture>
</p>

A Rust-based CLI tool for comprehensive WordPress diagnostics and maintenance.

WP Agent automates common WordPress maintenance checks, helping you identify potential issues with your site setup quickly and efficiently.

<p align="center">
  <img src="./docs/demo.gif" alt="animated" width="80%" />
</p>

## Features

- **Automatic WP-CLI management**: Detects and installs WP-CLI if needed
- **Smart root detection**: Automatically finds your WordPress installation
- **Root execution support**: Handles execution as root seamlessly
- **Comprehensive diagnostics**:
  - WP-CLI health check: verifies wp-cli can actually boot WordPress and flags a stale wp-cli phar
  - Database integrity and optimization checks
  - Plugin updates, security audits, and redundant-plugin detection (e.g. duplicate security scanners or image optimizers)
  - PHP patch status: the PHP the web server runs and wp-cli's PHP vs. php.net's latest release per branch, end-of-life PHP branches and Ubuntu releases
  - System resource monitoring
  - Network connectivity verification
  - Security checksums and debug mode detection
  - Compromise detection: `LD_PRELOAD` rootkits, processes and files hidden from `ps`/`ls`, binaries running from temp dirs, extra `dockerd` daemons, PHP webshells in uploads
  - Exposure: database dumps, archives and export-plugin files publicly downloadable from uploads; administrators/editors registered in the last 90 days
  - Mail: no SMTP plugin with direct Postfix delivery (rejected by Gmail/Outlook), missing SASL password map on a relay, messages stuck in the Postfix queue
  - Performance analysis (object cache incl. LiteSpeed/Memcached, autoloaded options, wp-cron vs. system cron, PHP memory_limit/max_execution_time, image-upload CPU risk on constrained hardware)
  - Error log scanning for PHP fatal errors and warnings, ranked by offending plugin, plus noisy-non-fatal-plugin flagging
  - Scanner/bot traffic detection from the access log (bursty 404s, known-malicious-path hits), fail2ban presence check, and CPU load-vs-traffic correlation
  - Maintenance checks (revisions, transients, debug logs)
- **Resilient to a broken wp-cli**: falls back to a direct database query when wp-cli itself fails, logging wp-cli's original error alongside the fallback
- **Color-coded reports**: Easy-to-read summary with OK/WARNING/ERROR indicators
- **JSON output and exit codes**: `--json` for monitoring and CI, non-zero exit when any check reports ERROR

## Documentation

- [Quickstart](#quickstart)
- [Installation](./docs/installation.md)
- [Basic Usage](./docs/basic-usage.md)
- [Features](./docs/features.md)
- [Creating Releases](./docs/creating-releases.md)

## Quickstart

Install WP Agent with a single command:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/candidosales/wp-agent-tool/releases/latest/download/install.sh | sh
```

Then navigate to your WordPress directory and run:

```bash
wp-agent
```

The tool will automatically:

1. Check for WP-CLI (and install it if needed)
2. Locate your WordPress root
3. Run comprehensive diagnostics
4. Display a summary report

### JSON output and exit codes

```bash
wp-agent --json > report.json   # report on stdout, progress on stderr
```

| Exit code | Meaning |
|-----------|---------|
| `0` | Ran; no module reported ERROR (warnings allowed) |
| `1` | Could not run (WP-CLI or WordPress root not found) |
| `2` | Ran; at least one module reported ERROR |

For more installation options, see the [Installation Guide](./docs/installation.md).

## Prerequisites

- **Unix-like Environment**: Tested on macOS and Linux
- **WP-CLI** (Optional): The tool can install a local copy if not found

## Development

### Project structure

```
src/
├── main.rs              # Entry point and orchestration
├── cli.rs               # Command-line interface
├── wp.rs                # WP-CLI wrapper
├── report.rs            # Report generation and display
└── diagnosis/           # Diagnostic modules
    ├── database.rs
    ├── plugins.rs
    ├── system.rs
    ├── network.rs
    ├── security.rs
    ├── performance.rs
    ├── maintenance.rs
    ├── errorlog.rs
    ├── scanner.rs
    ├── patches.rs
    ├── compromise.rs
    ├── exposure.rs
    ├── mail.rs
    └── wpcli_health.rs

e2e/                     # Docker end-to-end tests, reports in e2e/artifacts/
```

### Building from source

```bash
# Clone the repository
git clone https://github.com/candidosales/wp-agent-tool
cd wp-agent-tool

# Build the release binary
cargo build --release

# Run tests
cargo test

# End-to-end tests (need Docker; patches.sh also needs network)
./e2e/compromise.sh
./e2e/patches.sh
./e2e/exposure-mail.sh
./e2e/wpcli-health.sh

# The binary will be at target/release/wp-agent
```

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development guidelines.

## License

[MIT](./LICENSE)

## Contributing

Contributions are welcome! Please read our [Contributing Guide](./CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.
