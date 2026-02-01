# Installation guide

## Quickstart

The fastest way to install WP Agent is using our installation script:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/candidosales/wp-agent-tool/releases/latest/download/install.sh | sh
```

Then restart your shell and run:

```bash
wp-agent --help
```

## Installation methods

### Script installation (recommended)

The installation script automatically:

- Detects your OS and architecture
- Downloads the correct binary
- Installs to `~/.local/bin` (or custom location)
- Makes the binary executable

```bash
# Default installation
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/candidosales/wp-agent-tool/releases/latest/download/install.sh | sh

# Custom installation directory
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/candidosales/wp-agent-tool/releases/latest/download/install.sh | INSTALL_DIR=/usr/local/bin sh
```

Make sure `~/.local/bin` is in your PATH:

```bash
# Add to ~/.bashrc, ~/.zshrc, or equivalent
export PATH="$HOME/.local/bin:$PATH"
```

### Manual binary installation

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

### Building from source

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

#### Cross-compilation (macOS to Linux)

If you're developing on macOS and need to deploy to a Linux server:

1. **Add Linux target**:

   ```bash
   rustup target add x86_64-unknown-linux-musl
   ```

2. **Install linker** (using Homebrew):

   ```bash
   brew install messense/macos-cross-toolchains/x86_64-unknown-linux-musl
   ```

3. **Build**:

   ```bash
   cargo build --release --target x86_64-unknown-linux-musl
   ```

   The static binary will be at `target/x86_64-unknown-linux-musl/release/wp-agent`.

## Verifying installation

After installation, verify it works:

```bash
wp-agent --version
wp-agent --help
```

## Next steps

- Read the [Basic usage guide](./basic-usage.md)
- Check out the [Features documentation](./features.md)
- Learn about [Contributing](../CONTRIBUTING.md)
