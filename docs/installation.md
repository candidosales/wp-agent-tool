# Installation Guide

## Quickstart

The fastest way to install WP Agent is using our installation script:

```bash
curl -sSf https://github.com/candidosales/wp-agent-tool/releases/latest/download/install.sh | sh
```

Then restart your shell and run:

```bash
wp-agent --help
```

## Installation Methods

### Script Installation (Recommended)

The installation script automatically:

- Detects your OS and architecture
- Downloads the correct binary
- Installs to `~/.local/bin` (or custom location)
- Makes the binary executable

```bash
# Default installation
curl -sSf https://github.com/candidosales/wp-agent-tool/releases/latest/download/install.sh | sh

# Custom installation directory
curl -sSf https://github.com/candidosales/wp-agent-tool/releases/latest/download/install.sh | INSTALL_DIR=/usr/local/bin sh
```

Make sure `~/.local/bin` is in your PATH:

```bash
# Add to ~/.bashrc, ~/.zshrc, or equivalent
export PATH="$HOME/.local/bin:$PATH"
```

### Manual Binary Installation

Download the appropriate binary for your platform from the [releases page](https://github.com/candidosales/wp-agent-tool/releases):

#### Linux

```bash
wget https://github.com/candidosales/wp-agent-tool/releases/latest/download/wp-agent-x86_64-unknown-linux-gnu.tar.gz
tar -xzf wp-agent-x86_64-unknown-linux-gnu.tar.gz
sudo mv wp-agent /usr/local/bin/
```

#### macOS (Intel)

```bash
wget https://github.com/candidosales/wp-agent-tool/releases/latest/download/wp-agent-x86_64-apple-darwin.tar.gz
tar -xzf wp-agent-x86_64-apple-darwin.tar.gz
sudo mv wp-agent /usr/local/bin/
```

#### macOS (Apple Silicon)

```bash
wget https://github.com/candidosales/wp-agent-tool/releases/latest/download/wp-agent-aarch64-apple-darwin.tar.gz
tar -xzf wp-agent-aarch64-apple-darwin.tar.gz
sudo mv wp-agent /usr/local/bin/
```

#### Windows

1. Download `wp-agent-x86_64-pc-windows-msvc.zip` from the [releases page](https://github.com/candidosales/wp-agent-tool/releases)
2. Extract `wp-agent.exe`
3. Move it to a directory in your PATH

### Building from Source

If you have Rust installed, you can build from source:

```bash
# Clone the repository
git clone https://github.com/candidosales/wp-agent-tool
cd wp-agent-tool

# Build the release binary
cargo build --release

# The binary will be at target/release/wp-agent
sudo mv target/release/wp-agent /usr/local/bin/
```

#### Cross-Compilation (macOS to Linux)

If you're developing on macOS and need to deploy to a Linux server:

1. **Add Linux Target**:

   ```bash
   rustup target add x86_64-unknown-linux-musl
   ```

2. **Install Linker** (using Homebrew):

   ```bash
   brew install messense/macos-cross-toolchains/x86_64-unknown-linux-musl
   ```

3. **Build**:

   ```bash
   cargo build --release --target x86_64-unknown-linux-musl
   ```

   The static binary will be at `target/x86_64-unknown-linux-musl/release/wp-agent`.

## Verifying Installation

After installation, verify it works:

```bash
wp-agent --version
wp-agent --help
```

## Next Steps

- Read the [Basic Usage Guide](./basic-usage.md)
- Check out the [Features Documentation](./features.md)
- Learn about [Contributing](../CONTRIBUTING.md)
