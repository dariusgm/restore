# Windows Restore

A Rust utility to extract and restore Windows backup ZIP files while preserving folder structure.

## Features

- **Recursive ZIP Discovery**: Automatically finds all ZIP files in a directory and subdirectories
- **Natural Sorting**: Sorts ZIP files using natural number ordering (e.g., backup1.zip, backup2.zip, backup10.zip)
- **Drive Letter Stripping**: Automatically removes Windows drive letters (C:/, D:\, etc.) from file paths
- **Analysis Mode**: Preview backup contents without extracting
- **Error Handling**: Comprehensive error reporting and recovery
- **Progress Display**: Real-time progress updates during extraction

## Usage

### Basic Extraction

```bash
cargo run -- --source /path/to/backups --dest /path/to/restore
```

### Analyze Only (Preview without extracting)

```bash
cargo run -- --source /path/to/backups --analyze-only
```

### Command-line Options

- `-s, --source <PATH>`: Path to the backup folder containing ZIP files (required)
- `-d, --dest <PATH>`: Destination path for restored files (required unless using `--analyze-only`)
- `-a, --analyze-only`: Analyze backups without extracting files

## How It Works

1. **Scan Phase**: Recursively searches for all `.zip` files in the source directory
2. **Sort Phase**: Sorts files using natural ordering to ensure correct extraction sequence
3. **Analyze Phase**: Displays backup statistics and file type distribution
4. **Confirmation**: Prompts user before starting extraction
5. **Extract Phase**: Extracts files while:
   - Removing drive letters from paths
   - Creating necessary directory structures
   - Handling errors gracefully
   - Reporting progress and statistics

## Building

```bash
cargo build --release
```

The compiled binary will be available in `target/release/restore`.

## Example Output

### Analysis Phase
```
============================================================
 Windows Backup Analyzer
============================================================
 Source directory:  /mnt/backups
 ZIP files:         5
 Total size:        45.23 GB

 Sample from: backup_001.zip
   .jpg        -> 3421 files
   .docx       -> 156 files
   .xlsx       -> 89 files
   .pdf        -> 234 files
   .txt        -> 1203 files
============================================================
```

### Extraction Phase
```
[1/5] backup_001.zip... 12543 files
[2/5] backup_002.zip... 8921 files
[3/5] backup_003.zip... 9654 files
[4/5] backup_004.zip... 11234 files
[5/5] backup_005.zip... 7342 files

============================================================
 Extraction completed!
 Files extracted:   49694
 Errors:            0
 Destination:       /mnt/restored
============================================================
```

## Requirements

- Rust 1.56 or later
- Dependencies are managed via Cargo.toml

## GitHub Actions Workflows

This project includes automated GitHub Actions workflows for building and testing:

### Release Workflow (`release.yml`)

Automatically creates binary artifacts when you push a Git tag (e.g., `v1.0.0`):

- **Trigger**: Git tags matching `v*` pattern or manual workflow dispatch
- **Platforms**: 
  - Linux (x86_64)
  - Windows (x86_64)
  - macOS (x86_64 and ARM64)
- **Artifacts**: Stored for 30 days
- **Release**: Automatically creates GitHub Release with binaries attached

**Usage:**

```bash
git tag v1.0.0
git push origin v1.0.0
```

### Build Workflow (`build.yml`)

Runs on every push to `main`/`develop` branches and pull requests:

- **Builds**: All platforms in release mode
- **Tests**: Runs `cargo test --release`
- **Linting**: Runs `cargo clippy` with strict warnings
- **Caching**: Efficiently caches dependencies to speed up builds
- **Artifacts**: Stored for 30 days (for inspection if needed)

## Binary Downloads

Pre-compiled binaries are available in two ways:

1. **GitHub Releases**: Download from the [Releases](../../releases) page (for tagged versions)
2. **Workflow Artifacts**: Download from completed workflow runs (latest builds)

Binary naming convention:
- Linux: `restore-linux-x86_64`
- Windows: `restore-windows-x86_64.exe`
- macOS Intel: `restore-macos-x86_64`
- macOS Apple Silicon: `restore-macos-aarch64`

## License

See LICENSE file for details.
