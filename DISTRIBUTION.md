# Distribution Guide

## Overview

This project supports multiple distribution methods for different platforms:
- Command-line tools (TUI and backend)
- Desktop applications (macOS, Windows, Linux)
- Mobile applications (Android, iOS)

## Automated Release Process

### Using cargo-dist

Releases are automatically handled by cargo-dist when you push a version tag:

```bash
# Tag a release
git tag v0.1.0
git push origin v0.1.0
```

This will trigger GitHub Actions to:
1. Create a GitHub Release
2. Build binaries for all platforms
3. Generate installers (shell, PowerShell)
4. Upload artifacts to the release

## Manual Build Scripts

For local development and testing, use the provided build scripts:

### Build All Platforms
```bash
./scripts/build-all.sh
```

### Platform-Specific Builds

#### Android APK
```bash
./scripts/build-android.sh
```
Output: `dist/android/*.apk`

#### macOS Bundle
```bash
./scripts/build-macos.sh
```
Output: `dist/macos/Graph Learning.app` and `.dmg`

#### Windows Installer
```powershell
# Run in PowerShell on Windows
.\scripts\build-windows.ps1
```
Output: `dist\windows\GraphLearning.exe` and `.msi` (if WiX is installed)

#### Linux AppImage
```bash
./scripts/build-linux-appimage.sh
```
Output: `dist/linux/GraphLearning-x86_64.AppImage`

## Binary Artifacts

### CLI Tools
- **graph-learning-cli**: Terminal UI for the learning system
- **graph-learning-backend**: Backend server for data collection

### Desktop Applications
- **xilem_example_desktop**: Cross-platform desktop application using Xilem

### Mobile Applications
- **xilem_example_mobile**: Android/iOS application

## Distribution Channels

### Direct Download
All binaries are available from GitHub Releases after tagging a version.

### Package Managers (Future)
The configuration supports future distribution via:
- Homebrew (macOS/Linux)
- npm (cross-platform)
- System package managers (apt, yum, etc.)

## Architecture Support

The build system supports multiple architectures:
- x86_64 (Intel/AMD 64-bit)
- aarch64 (ARM 64-bit, including Apple Silicon)
- Multiple Linux variants (GNU, musl)

## Requirements

### For Building
- Rust toolchain (stable)
- cargo-dist (v0.0.4)
- Platform-specific tools:
  - Android: cargo-apk, Android SDK
  - macOS: Xcode Command Line Tools
  - Windows: Visual Studio Build Tools, WiX Toolset (optional)
  - Linux: appimagetool

### For Running
- Minimal system requirements per platform
- No additional runtime dependencies (statically linked)

## Troubleshooting

### Build Failures
1. Ensure all dependencies are installed
2. Check Rust toolchain is up to date: `rustup update`
3. For cross-compilation, install appropriate targets:
   ```bash
   rustup target add aarch64-unknown-linux-gnu
   rustup target add x86_64-pc-windows-msvc
   # etc.
   ```

### Distribution Issues
1. Verify version tags follow the pattern: `v[0-9]+.[0-9]+.[0-9]+`
2. Check GitHub Actions logs for CI failures
3. Ensure GitHub repository has appropriate permissions for releases

## Version Management

Update versions in:
1. Root `Cargo.toml` (workspace version)
2. Individual package `Cargo.toml` files if needed
3. Tag the commit with the matching version

## Security Considerations

- All binaries are built in CI with reproducible builds
- Code signing is recommended for production releases
- Use appropriate entitlements for macOS/iOS
- Follow platform-specific security guidelines