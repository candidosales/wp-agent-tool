# Creating a Release

This guide explains how to create and publish releases for WP Agent.

## Prerequisites

Before creating a release, ensure:

- All changes are committed and pushed to the `main` branch
- Tests are passing
- Version number is updated in `Cargo.toml` (if applicable)
- CHANGELOG or release notes are prepared

## Release Process

### 1. Prepare the Release

Review your changes and decide on the version number following [Semantic Versioning](https://semver.org/):

- **MAJOR** version (v2.0.0): Incompatible API changes
- **MINOR** version (v1.1.0): New functionality, backwards compatible
- **PATCH** version (v1.0.1): Bug fixes, backwards compatible

### 2. Create and Push a Git Tag

```bash
# Create a tag with the version number (must start with 'v')
git tag v0.1.0

# Push the tag to GitHub
git push origin v0.1.0
```

> **Note**: The tag MUST start with `v` (e.g., `v0.1.0`, `v1.2.3`) to trigger the release workflow.

### 3. Monitor the Release Workflow

Once you push the tag, GitHub Actions will automatically:

1. **Build binaries** for all supported platforms:
   - Linux (x86_64-unknown-linux-gnu)
   - macOS Intel (x86_64-apple-darwin)
   - macOS Apple Silicon (aarch64-apple-darwin)

2. **Create archives**:
   - `.tar.gz` files for Unix systems

3. **Generate checksums** (SHA256) for all archives

4. **Create the installer script** (`install.sh`)

5. **Create a GitHub Release** with:
   - Auto-generated release notes from PRs and commits
   - All binary archives
   - Checksum files
   - Installer script

### 4. Monitor Progress

1. Go to the **Actions** tab in your GitHub repository
2. Find the "Release" workflow run
3. Monitor the build progress for each platform
4. Ensure all jobs complete successfully

### 5. Verify the Release

Once the workflow completes:

1. Go to the **Releases** section of your repository
2. Verify the new release appears with:
   - Correct version number
   - Release notes listing PRs and changes
   - All binary archives (4 platforms)
   - Checksum files
   - `install.sh` script

### 6. Test the Release

Test the installation script:

```bash
# Test the curl installer
curl -sSf https://github.com/candidosales/wp-agent-tool/releases/download/v0.1.0/install.sh | sh

# Verify installation
wp-agent --version
```

Test manual downloads for different platforms if possible.

## Creating a Pre-release

For testing or beta versions, you can create a pre-release:

```bash
# Create a pre-release tag
git tag v0.1.0-beta.1
git push origin v0.1.0-beta.1
```

After the workflow completes, edit the release on GitHub and mark it as a "Pre-release".

## Troubleshooting

### Build Failures

If a build fails for a specific platform:

1. Check the Actions logs for error details
2. Common issues:
   - Dependency compilation errors
   - Platform-specific code issues
   - Missing system libraries

### Tag Already Exists

If you need to recreate a tag:

```bash
# Delete local tag
git tag -d v0.1.0

# Delete remote tag
git push origin :refs/tags/v0.1.0

# Create new tag
git tag v0.1.0
git push origin v0.1.0
```

> **Warning**: Deleting and recreating tags is not recommended for published releases.

### Release Not Created

If the workflow runs but doesn't create a release:

1. Check that the tag starts with `v`
2. Verify the workflow has `contents: write` permissions
3. Check the workflow logs for errors in the release step

## Manual Release (Advanced)

If you need to create a release manually:

### Build Locally

```bash
# Build for your current platform
cargo build --release

# The binary will be at target/release/wp-agent
```

### Cross-compile for Other Platforms

```bash
# Add targets
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-msvc

# Build for specific target
cargo build --release --target x86_64-unknown-linux-gnu
```

### Create Release on GitHub

1. Go to **Releases** → **Draft a new release**
2. Choose or create a tag
3. Fill in the release title and description
4. Upload the built binaries
5. Publish the release

## Best Practices

### Before Releasing

- [ ] Run all tests: `cargo test`
- [ ] Check for linting issues: `cargo clippy`
- [ ] Update version in `Cargo.toml` if needed
- [ ] Update documentation
- [ ] Review and update CHANGELOG

### Version Numbering

- Use semantic versioning (MAJOR.MINOR.PATCH)
- Start with `v0.1.0` for initial releases
- Increment appropriately based on changes
- Use pre-release tags for testing (e.g., `v1.0.0-beta.1`)

### Release Notes

The GitHub Actions workflow automatically generates release notes from:

- Pull request titles and descriptions
- Commit messages since the last release

To improve auto-generated notes:

- Write clear, descriptive PR titles
- Use conventional commit messages
- Group related changes in single PRs

### Testing Releases

Before announcing a release:

- Test the curl installer on a clean system
- Verify binaries work on target platforms
- Check that all download links are accessible
- Review the auto-generated release notes for clarity

## Example Workflow

Here's a complete example of creating a release:

```bash
# 1. Ensure you're on main and up to date
git checkout main
git pull origin main

# 2. Run tests
cargo test

# 3. Update version (if needed)
# Edit Cargo.toml and update version = "0.1.0"

# 4. Commit version change
git add Cargo.toml
git commit -m "chore: bump version to 0.1.0"
git push origin main

# 5. Create and push tag
git tag v0.1.0
git push origin v0.1.0

# 6. Monitor GitHub Actions
# Visit: https://github.com/candidosales/wp-agent-tool/actions

# 7. Verify release
# Visit: https://github.com/candidosales/wp-agent-tool/releases

# 8. Test installation
curl -sSf https://github.com/candidosales/wp-agent-tool/releases/latest/download/install.sh | sh
wp-agent --version
```

## Related Documentation

- [GitHub Actions Workflow](../.github/workflows/release.yml)
- [Installation Guide](./installation.md)
- [Contributing Guide](../CONTRIBUTING.md)
