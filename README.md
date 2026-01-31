# WP Agent Tool

WP Agent is a Rust-based CLI tool designed to diagnose and analyze WordPress installations. It automates common maintenance checks and helps identify potential issues with your WordPress site setup.

## Features

- **WP-CLI Management**: Automatically checks for `wp-cli` and offers to install it locally if missing.
- **Intelligent Root Detection**: Automatically locates your WordPress root directory or prompts if it cannot be found.
- **Root Execution Support**: Seamlessly handles execution as root by switching to the site owner or using `--allow-root` automatically.
- **Comprehensive Diagnosis**:
  - **Database**: Runs integrity checks (`wp db check`) and analyzes database size.
  - **Plugins**: Identifies plugins with available updates and lists inactive plugins.
  - **System**: Checks PHP version and disk usage (specifically `/tmp` or root partition).
  - **Network**: Verifies external connectivity (Google) and internal site reachability.
- **Summary Reporting**: key issues are highlighted in a color-coded summary report.

## Prerequisites

- **Rust**: You need to have Rust and Cargo installed to build the project.
- **WP-CLI** (Optional): The tool can install a local copy if one is not found in your system path.
- **Unix-like Environment**: Tested on macOS/Linux.

## Installation & Building

```bash
# Clone the repository
git clone <repository-url>
cd wp-agent-tool

# Build the release binary
cargo build --release
```

The compiled binary will be available at `target/release/wp-agent`.

## Building for Different Platforms

### macOS & Linux

To build natively on macOS or Linux, simply run:

```bash
cargo build --release
```

The binary will be located at `target/release/wp-agent`.

### Windows

To build natively on Windows (via PowerShell or Command Prompt):

```bash
cargo build --release
```

The executable will be located at `target\release\wp-agent.exe`.

### Cross-Compilation (Mac to Linux)

If you are developing on macOS and need to deploy to a Linux server (e.g., Digital Ocean VPS), you can cross-compile:

1.  **Add Linux Target**:
    ```bash
    rustup target add x86_64-unknown-linux-musl
    ```
2.  **Install Linker** (using Homebrew):
    ```bash
    brew install messense/macos-cross-toolchains/x86_64-unknown-linux-musl
    brew install messense/macos-cross-toolchains/aarch64-unknown-linux-gnu
    ```
3.  **Build**:
    ```bash
    cargo build --release --target x86_64-unknown-linux-musl
    ```
    The static binary will be at `target/x86_64-unknown-linux-musl/release/wp-agent`.

## Usage

Navigate to your WordPress project directory (or any directory, if you want to specify the path manually) and run:

```bash
# Run from the project root
cargo run

# Or run the built binary
./target/release/wp-agent
```

### Options

```bash
wp-agent --help
```

## How It Works

1.  **Initialization**: The tool starts and checks if `wp-cli` is available.
2.  **Root Search**: It looks for `wp-config.php` in the current directory and parent directories.
3.  **Diagnosis**: It runs a series of checks using `wp-cli` and system utilities.
4.  **Report**: It outputs a table illustrating the status of each module (OK, WARNING, ERROR) and details for any issues found.

## Development

Project structure:

- `src/main.rs`: Entry point and orchestration.
- `src/wp.rs`: Wrapper for `wp-cli` interactions.
- `src/diagnosis/`: Modules for specific checks (Database, Plugins, System, Network).
- `src/report.rs`: UI logic for displaying results.

## License

[MIT]
