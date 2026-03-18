# clipd

A clipboard daemon that watches the clipboard and transforms its contents using configurable rules. Useful for automatically cleaning up URLs, redirecting domains, or normalising any text pattern you copy.

## How it works

clipd polls the clipboard on a configurable interval and applies a list of **strategies** in order. The first strategy that produces a change wins; the result is written back to the clipboard.

Two strategy types are available:

| Type | What it does |
|------|-------------|
| `domain` | Replaces the host of a URL (`x.com` → `stupidpenisx.com`) |
| `match` | Applies a regex substitution with capture-group support |

## Default rules

Out of the box clipd ships with two rules:

- Redirect `x.com` links to `stupidpenisx.com`
- Normalise YouTube URLs to `https://youtube.com/watch?v=<id>` (strips tracking params)

## Install from release

Downloads the latest pre-built binary from GitHub Releases — no Rust toolchain required.

### macOS

Detects architecture (Apple Silicon or Intel) automatically.

```sh
./scripts/install_release_macos.sh
```

**Options** (set as environment variables):

| Variable | Default | Description |
|----------|---------|-------------|
| `BINARY_DEST` | `/usr/local/bin/clipd` | Where to install the binary |
| `LABEL` | `dev.thmsn.clipd` | launchd agent label |
| `LOG_DIR` | `/tmp` | Directory for stdout/stderr logs |

### Linux (systemd)

```sh
./scripts/install_release_linux.sh
```

**Options**:

| Variable | Default | Description |
|----------|---------|-------------|
| `BINARY_DEST` | `/usr/local/bin/clipd` | Where to install the binary |
| `SERVICE_NAME` | `clipd` | systemd unit name |

### Windows

Requires an Administrator prompt. Run one of:

```bat
scripts\InstallRelease.bat
```

```powershell
.\scripts\InstallRelease.ps1 -BinaryDest "C:\Program Files\clipd\clipd.exe" -TaskName clipd
```

> **Note:** The Windows and Linux clipboard backends are not yet implemented. These scripts are provided for future support.

## Build and install from source

Requires a Rust toolchain (`cargo`).

### macOS

```sh
./scripts/install_macos.sh
```

**Options** (set as environment variables):

| Variable | Default | Description |
|----------|---------|-------------|
| `BINARY_DEST` | `/usr/local/bin/clipd` | Where to install the binary |
| `LABEL` | `dev.thmsn.clipd` | launchd agent label |
| `LOG_DIR` | `/tmp` | Directory for stdout/stderr logs |

```sh
BINARY_DEST=~/.local/bin/clipd LOG_DIR=~/.local/share/clipd ./scripts/install_macos.sh
```

Registers a launchd user agent that starts at login and restarts on failure.

### Linux (systemd)

```sh
./scripts/install_systemd.sh
```

**Options**:

| Variable | Default | Description |
|----------|---------|-------------|
| `BINARY_DEST` | `/usr/local/bin/clipd` | Where to install the binary |
| `SERVICE_NAME` | `clipd` | systemd unit name |

Installs to `~/.config/systemd/user/` and enables the service for the current user session.

### Windows

Requires an Administrator prompt. Run one of:

```bat
scripts\Install.bat
```

```powershell
.\scripts\Install.ps1 -BinaryDest "C:\Program Files\clipd\clipd.exe" -TaskName clipd
```

Registers a Task Scheduler task that runs clipd at logon.

## Configuration

clipd uses [`libconfig`](https://github.com/charliethomson/libconfig) for configuration. The config file lives at the platform default path for `dev.thmsn.clipd`.

Example config:

```toml
tick_interval_ms = 100

[[patterns]]
style = "domain"
source = "x.com"
target = "nitter.net"

[[patterns]]
style = "match"
pattern = "^(.+)v=([a-zA-Z0-9-_]{11})(.+)$"
replacement = "https://youtube.com/watch?v=$2"

[[patterns]]
style = "nop"
```

Patterns are evaluated in order; the first one that changes the clipboard content wins.

## Usage

```
clipd [OPTIONS]

Options:
  -v, --verbose  Enable debug logging
  -d, --daemon   Run continuously (poll clipboard on interval)
  -h, --help     Print help
```

Without `--daemon`, clipd checks the clipboard once and exits.

## Building from source

```sh
cargo build --release
```

The binary is placed at `target/release/clipd`.

## Development

```sh
# Run tests
cargo test

# Run with coverage (installs cargo-tarpaulin if needed)
./scripts/cov.sh

# Pass extra flags to tarpaulin
./scripts/cov.sh --fail-under 80
```

## Releases

A release is created automatically on every push to `main`. Each release is tagged `v{version}` (bumped by commit message prefix) and includes pre-built binaries for:

- `aarch64-apple-darwin` (macOS ARM)
- `x86_64-apple-darwin` (macOS Intel)
- `x86_64-unknown-linux-gnu` (Linux)
- `x86_64-pc-windows-msvc` (Windows)
