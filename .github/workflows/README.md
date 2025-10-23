# GitHub Actions Workflows

This directory contains the automated CI/CD workflows for the gytmdl-gui project. These workflows handle building, testing, security scanning, and releasing the application across multiple platforms.

## Workflows Overview

### 1. Build and Release (`build-and-release.yml`)

**Triggers:**
- Push to tags matching `v*` (e.g., `v1.0.0`)
- Push to `main` branch
- Pull requests to `main` branch
- Manual workflow dispatch

**Purpose:**
- Builds sidecar binaries for all target platforms
- Builds Tauri application with bundled sidecars
- Creates platform-specific installers
- Handles code signing for Windows and macOS
- Creates GitHub releases with artifacts
- Tests installer functionality

**Platforms Supported:**
- Windows (x64, x86)
- macOS (Intel, Apple Silicon)
- Linux (x64, ARM64)

**Artifacts Created:**
- `.msi` installers for Windows
- `.dmg` files for macOS
- `.deb`, `.rpm`, and `.AppImage` files for Linux
- Checksums and metadata files

### 2. Dependency Updates (`dependency-update.yml`)

**Triggers:**
- Weekly schedule (Sundays at 2 AM UTC)
- Manual workflow dispatch

**Purpose:**
- Checks for updates to npm, Cargo, and gytmdl dependencies
- Creates automated pull requests with dependency updates
- Runs tests to validate updates
- Provides detailed update information

**Features:**
- Selective update types (npm, cargo, gytmdl, or all)
- Automated testing of updates
- Detailed PR descriptions with change summaries

### 3. Security Scan (`security-scan.yml`)

**Triggers:**
- Daily schedule (3 AM UTC)
- Push to `main` branch
- Pull requests to `main` branch
- Manual workflow dispatch

**Purpose:**
- Scans dependencies for known vulnerabilities
- Performs static code analysis with CodeQL
- Checks license compliance
- Monitors supply chain security
- Generates comprehensive security reports

**Scans Performed:**
- npm audit for JavaScript dependencies
- cargo audit for Rust dependencies
- License compliance checking
- Supply chain vulnerability analysis
- Code security analysis

### 4. Release Management (`release-management.yml`)

**Triggers:**
- Manual workflow dispatch only

**Purpose:**
- Automates the release preparation process
- Handles version bumping across all files
- Generates changelogs
- Creates and pushes git tags
- Validates release consistency
- Creates draft GitHub releases

**Release Types:**
- `patch` - Bug fixes (1.0.0 → 1.0.1)
- `minor` - New features (1.0.0 → 1.1.0)
- `major` - Breaking changes (1.0.0 → 2.0.0)
- `prerelease` - Alpha/beta/rc versions

## Configuration

### Secrets Required

The workflows require the following GitHub secrets to be configured:

#### Code Signing (Optional but Recommended)
- `WINDOWS_CERTIFICATE` - Base64 encoded Windows code signing certificate
- `WINDOWS_CERTIFICATE_PASSWORD` - Password for Windows certificate
- `APPLE_CERTIFICATE` - Base64 encoded Apple Developer certificate
- `APPLE_CERTIFICATE_PASSWORD` - Password for Apple certificate
- `APPLE_SIGNING_IDENTITY` - Apple signing identity name
- `APPLE_ID` - Apple ID for notarization
- `APPLE_PASSWORD` - App-specific password for Apple ID
- `APPLE_TEAM_ID` - Apple Developer Team ID

#### Tauri Updater (Optional)
- `TAURI_PRIVATE_KEY` - Private key for Tauri updater
- `TAURI_KEY_PASSWORD` - Password for Tauri private key

### Environment Variables

The workflows use the following environment variables:
- `CARGO_TERM_COLOR=always` - Enables colored Cargo output
- `TAURI_PRIVATE_KEY` - Set from secrets for updater functionality
- `TAURI_KEY_PASSWORD` - Set from secrets for updater functionality

## Usage

### Creating a Release

1. **Automated Release (Recommended):**
   ```bash
   # Go to GitHub Actions tab
   # Select "Release Management" workflow
   # Click "Run workflow"
   # Choose release type (patch/minor/major/prerelease)
   # Click "Run workflow"
   ```

2. **Manual Release:**
   ```bash
   # Update version in package.json, Cargo.toml, and tauri.conf.json
   git add .
   git commit -m "chore: bump version to 1.0.0"
   git tag -a "v1.0.0" -m "Release v1.0.0"
   git push origin main
   git push origin v1.0.0
   ```

### Testing Changes

Pull requests automatically trigger:
- Build validation
- Test execution
- Security scanning
- Installer testing

### Monitoring Security

Security scans run automatically and:
- Create artifacts with scan results
- Comment on pull requests with findings
- Generate comprehensive security reports
- Alert on critical vulnerabilities

### Updating Dependencies

Dependencies are automatically checked weekly and:
- Create pull requests for available updates
- Run tests to validate updates
- Provide detailed change information
- Can be triggered manually for immediate updates

## Troubleshooting

### Build Failures

1. **Sidecar Build Issues:**
   - Check that gytmdl repository is accessible
   - Verify PyInstaller is working correctly
   - Check Python version compatibility

2. **Code Signing Issues:**
   - Verify certificates are valid and not expired
   - Check that secrets are properly configured
   - Ensure signing identities match certificates

3. **Cross-Platform Issues:**
   - Check target platform compatibility
   - Verify system dependencies are installed
   - Review platform-specific build steps

### Release Issues

1. **Version Mismatch:**
   - Ensure all version files are updated consistently
   - Check that git tags match version numbers
   - Verify semantic versioning format

2. **Artifact Upload Issues:**
   - Check file sizes and formats
   - Verify GitHub token permissions
   - Review artifact retention settings

### Security Scan Issues

1. **False Positives:**
   - Review scan results manually
   - Update security tool configurations
   - Add exceptions for known safe dependencies

2. **License Compliance:**
   - Review problematic licenses
   - Consider alternative dependencies
   - Update license compatibility matrix

## Maintenance

### Regular Tasks

1. **Monthly:**
   - Review security scan results
   - Update workflow dependencies
   - Check code signing certificate expiration

2. **Quarterly:**
   - Review and update platform support
   - Evaluate new security tools
   - Update documentation

3. **Annually:**
   - Renew code signing certificates
   - Review workflow efficiency
   - Update security policies

### Workflow Updates

When updating workflows:
1. Test changes in a fork first
2. Use semantic versioning for action versions
3. Update documentation accordingly
4. Monitor first few runs after changes

## Support

For issues with the automated release pipeline:
1. Check workflow run logs in GitHub Actions
2. Review this documentation
3. Check the project's issue tracker
4. Contact the maintainers

## Security

These workflows handle sensitive operations including:
- Code signing with certificates
- Release artifact creation
- Dependency management
- Security scanning

Always:
- Keep secrets secure and rotate regularly
- Review workflow changes carefully
- Monitor for suspicious activity
- Follow security best practices