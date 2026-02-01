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
  - Database integrity and optimization checks
  - Plugin updates and security audits
  - PHP version and system resource monitoring
  - Network connectivity verification
  - Security checksums and debug mode detection
  - Performance analysis (object cache, autoloaded options, cron)
  - Maintenance checks (revisions, transients, debug logs)
- **Color-coded reports**: Easy-to-read summary with OK/WARNING/ERROR indicators

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
    └── maintenance.rs
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

# The binary will be at target/release/wp-agent
```

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development guidelines.

## License

[MIT](./LICENSE)

## Contributing

Contributions are welcome! Please read our [Contributing Guide](./CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.
