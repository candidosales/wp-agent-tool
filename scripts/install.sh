#!/bin/sh
set -e

# WP Agent installer script
# Usage: curl -sSf https://github.com/candidosales/wp-agent-tool/releases/latest/download/install.sh | sh

REPO="candidosales/wp-agent-tool"
BINARY_NAME="wp-agent"

# Colors for output
if [ -t 1 ]; then
  GREEN='\033[0;32m'
  BLUE='\033[0;34m'
  YELLOW='\033[1;33m'
  NC='\033[0m' # No Color
else
  GREEN=''
  BLUE=''
  YELLOW=''
  NC=''
fi

print_step() {
  printf "${BLUE}==>${NC} %b\n" "$1"
}

print_success() {
  printf "${GREEN}✓${NC} %b\n" "$1"
}

print_info() {
  printf "${YELLOW}ℹ${NC} %b\n" "$1"
}

get_latest_release() {
  curl --silent "https://api.github.com/repos/$REPO/releases/latest" |
    grep '"tag_name":' |
    sed -E 's/.*"([^"]+)".*/\1/'
}

detect_platform() {
  local os=$(uname -s | tr '[:upper:]' '[:lower:]')
  local arch=$(uname -m)

  case "$os" in
    linux*)
      case "$arch" in
        x86_64) echo "x86_64-unknown-linux-musl" ;;
        *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
      esac
      ;;
    darwin*)
      case "$arch" in
        x86_64) echo "x86_64-apple-darwin" ;;
        arm64) echo "aarch64-apple-darwin" ;;
        *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
      esac
      ;;
    *)
      echo "Unsupported OS: $os" >&2
      exit 1
      ;;
  esac
}

main() {
  echo ""
  echo "╔════════════════════════════════════════╗"
  echo "║      WP Agent Installer                ║"
  echo "╚════════════════════════════════════════╝"
  echo ""

  print_step "Detecting platform..."
  local platform=$(detect_platform)
  print_success "Detected: $platform"

  print_step "Fetching latest release..."
  local version=$(get_latest_release)
  print_success "Latest version: $version"

  local archive="wp-agent-${platform}.tar.gz"
  local url="https://github.com/$REPO/releases/download/$version/$archive"
  
  print_step "Downloading wp-agent $version..."
  local tmpdir=$(mktemp -d)
  cd "$tmpdir"
  
  if curl -sSfL "$url" -o "$archive"; then
    print_success "Download complete"
  else
    echo "Failed to download $url" >&2
    exit 1
  fi
  
  print_step "Extracting archive..."
  tar -xzf "$archive"
  print_success "Extraction complete"
  
  local install_dir="${INSTALL_DIR:-$HOME/.local/bin}"
  mkdir -p "$install_dir"
  
  print_step "Installing to $install_dir..."
  mv "$BINARY_NAME" "$install_dir/"
  chmod +x "$install_dir/$BINARY_NAME"
  print_success "Installation complete"
  
  cd - > /dev/null
  rm -rf "$tmpdir"
  
  echo ""
  echo "╔════════════════════════════════════════╗"
  echo "║   ${GREEN}✓${NC} wp-agent installed successfully!   ║"
  echo "╚════════════════════════════════════════╝"
  echo ""
  print_info "Binary location: $install_dir/$BINARY_NAME"
  
  # Check if directory is in PATH
  case ":$PATH:" in
    *":$install_dir:"*)
      print_success "$install_dir is already in your PATH"
      ;;
    *)
      # Add to PATH automatically
      local shell_config=""
      local shell_name=$(basename "$SHELL")
      local path_cmd="export PATH=\"$install_dir:\$PATH\""

      case "$shell_name" in
        zsh)
          shell_config="${ZDOTDIR:-$HOME}/.zshrc"
          ;;
        bash)
          if [ -f "$HOME/.bashrc" ]; then
            shell_config="$HOME/.bashrc"
          elif [ -f "$HOME/.bash_profile" ]; then
            shell_config="$HOME/.bash_profile"
          else
            shell_config="$HOME/.bashrc"
          fi
          ;;
        fish)
            shell_config="$HOME/.config/fish/conf.d/wp-agent.fish"
            path_cmd="fish_add_path $install_dir"
            mkdir -p "$(dirname "$shell_config")"
          ;;
        *)
          shell_config="$HOME/.profile"
          ;;
      esac

      if [ -n "$shell_config" ]; then
        if [ "$shell_name" = "fish" ]; then
            print_step "Adding to PATH in $shell_config..."
            echo "$path_cmd" > "$shell_config"
            print_success "Added to PATH"
            
            echo ""
            print_info "Get started with: ${GREEN}wp-agent --help${NC}"
            echo ""
            print_info "Restarting shell to apply changes..."
            sleep 1
            [ ! -t 0 ] && [ -e /dev/tty ] && exec < /dev/tty
            exec "$SHELL" -l
        else 
            if [ -f "$shell_config" ] && grep -q "$install_dir" "$shell_config"; then
              print_success "PATH already configured in $shell_config"
            else
              print_step "Adding to PATH in $shell_config..."
              echo "" >> "$shell_config"
              echo "# wp-agent" >> "$shell_config"
              echo "$path_cmd" >> "$shell_config"
              print_success "Added to PATH"
              
              echo ""
              print_info "Get started with: ${GREEN}wp-agent --help${NC}"
              echo ""
              print_info "Restarting shell to apply changes..."
              sleep 1
              [ ! -t 0 ] && [ -e /dev/tty ] && exec < /dev/tty
              exec "$SHELL" -l
            fi
        fi
      else
          print_info "Could not detect shell config file. Please add manually:"
          echo "    export PATH=\"$install_dir:\$PATH\""
      fi
      ;;
  esac
  
  echo ""
  print_info "Get started with: ${GREEN}wp-agent --help${NC}"
  echo ""
}

main
