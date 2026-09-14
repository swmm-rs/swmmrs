# Install the command-line runner

The standalone [`scripts/install.sh`](https://github.com/swmm-rs/swmmrs/blob/main/scripts/install.sh) and [`scripts/install.ps1`](https://github.com/swmm-rs/swmmrs/blob/main/scripts/install.ps1) installers live in the repository. By default, they query the GitHub Releases API for the newest stable `rs-*` release, download its `runswmmrs` binary, verify its SHA-256 checksum, and install it for the current user. Python-only `py-*` releases are ignored. The installers do not require the GitHub CLI, Rust, or administrator privileges.

## macOS and Linux

Use `curl` to download and run the installer:

```sh
curl -LsSf https://raw.githubusercontent.com/swmm-rs/swmmrs/main/scripts/install.sh | sh
```

If your system does not have `curl`, use `wget`:

```sh
wget -qO- https://raw.githubusercontent.com/swmm-rs/swmmrs/main/scripts/install.sh | sh
```

## Windows

From Windows PowerShell 5.1 or PowerShell 7, download the script first:

```powershell
Invoke-WebRequest -UseBasicParsing -Uri "https://raw.githubusercontent.com/swmm-rs/swmmrs/main/scripts/install.ps1" -OutFile "$HOME\Downloads\install-swmmrs.ps1"
```

Review the downloaded script before running it (see below), then execute it in a
fresh Windows PowerShell process:

```powershell
powershell -NoProfile -File "$HOME\Downloads\install-swmmrs.ps1"
```

`-NoProfile` skips shell startup customizations. This process does not pipe remote
code into `iex` or bypass execution policy. If Defender flags the script or your
execution policy blocks it, do not disable protection; use a manually downloaded
release ZIP if it is not flagged, or ask your administrator for approval.

## Inspect before installing

Reviewing a downloaded script before running it is safer than piping it directly to a shell.

### macOS and Linux

```sh
curl -LsSf -o install.sh https://raw.githubusercontent.com/swmm-rs/swmmrs/main/scripts/install.sh
less install.sh
sh install.sh
```

### Windows

```powershell
notepad "$HOME\Downloads\install-swmmrs.ps1"
```

You can also request a Defender scan of the downloaded script:

```powershell
Start-MpScan -ScanType CustomScan -ScanPath "$HOME\Downloads\install-swmmrs.ps1"
```

## Install a specific version

Set `SWMMRS_VERSION` to an `rs-*` release tag. A bare semantic version or legacy `v` prefix is also accepted and normalized to `rs-*`.

### macOS and Linux

```sh
curl -LsSf https://raw.githubusercontent.com/swmm-rs/swmmrs/main/scripts/install.sh | SWMMRS_VERSION=rs-0.1.0 sh
```

### Windows

```powershell
# Download and review the script as described above first.
$env:SWMMRS_VERSION = "rs-0.1.0"
powershell -NoProfile -File "$HOME\Downloads\install-swmmrs.ps1"
Remove-Item Env:SWMMRS_VERSION
```

## Installation location and PATH

The default destination on all supported systems is:

- macOS and Linux: `~/.local/bin/runswmmrs`
- Windows: `~\.local\bin\runswmmrs.exe`

The installer adds this directory to PATH when necessary. On macOS and Linux it updates the startup file for the detected shell; on Windows it updates the current user's PATH. Open a new terminal after installation. On Windows, close and reopen the terminal application; if an existing Windows session still has a stale PATH, sign out and back in.

Set `SWMMRS_NO_MODIFY_PATH=1` to install without changing PATH, or set `SWMMRS_INSTALL_DIR` to choose another destination.

### macOS and Linux

```sh
curl -LsSf https://raw.githubusercontent.com/swmm-rs/swmmrs/main/scripts/install.sh \
  | SWMMRS_NO_MODIFY_PATH=1 SWMMRS_INSTALL_DIR="$HOME/bin" sh
```

### Windows

```powershell
# Download and review the script as described above first.
$env:SWMMRS_NO_MODIFY_PATH = "1"
$env:SWMMRS_INSTALL_DIR = "$HOME\bin"
powershell -NoProfile -File "$HOME\Downloads\install-swmmrs.ps1"
Remove-Item Env:SWMMRS_NO_MODIFY_PATH, Env:SWMMRS_INSTALL_DIR
```

## Supported systems

| Operating system | Architectures | Release format |
| --- | --- | --- |
| Linux with glibc 2.35+ (Ubuntu 22.04+) | x86-64, ARM64 | `.tar.gz` |
| macOS 10.13+ (Intel), macOS 11+ (Apple silicon) | Intel, Apple silicon | `.tar.gz` |
| Windows | x86-64, ARM64 | `.zip` |

Alpine and other musl-based Linux systems are not currently supported by the standalone installer. Linux releases are built and smoke-tested natively on Ubuntu 22.04 runners; the installer rejects glibc older than 2.35. Release builds explicitly target the portable `x86-64` CPU baseline on x86-64 and the `generic` CPU baseline on ARM64. Intel macOS builds target 10.13 because the native parallel runtime uses C++17 aligned allocation; Apple silicon builds target macOS 11.


Each Rust release includes the executable, source-license scope, the permissive [swmmrs Binary Runtime License](../../LICENSES/SWMMRS-BINARY-RUNTIME.txt), and the license texts referenced by the third-party notices inside every CLI archive. Summary and runtime-license files are also published as common release assets. Each archive has a matching `.sha256` asset. The installer requires checksum tooling and refuses to install an archive that does not match its published checksum.

## Unsigned binary warnings

The release binaries are unsigned because public macOS and Windows signing requires paid developer services. The installers above still verify the downloaded archive's SHA-256 checksum before installing it. If that check fails, stop; do not bypass an operating-system warning.

### macOS

On first run, macOS may say that `runswmmrs` cannot be opened because the developer cannot be verified, or that Apple cannot check it for malicious software. To approve only this binary:

1. Run `runswmmrs --version` once so macOS records the block.
2. Open **System Settings > Privacy & Security**.
3. Under **Security**, click **Open Anyway** for `runswmmrs`, authenticate if asked, then run the command again.

Alternatively, remove quarantine from only the default installed binary:

```sh
xattr -d com.apple.quarantine "$HOME/.local/bin/runswmmrs"
"$HOME/.local/bin/runswmmrs" --version
```

Replace the path if you used `SWMMRS_INSTALL_DIR`. Do not disable Gatekeeper globally.

### Windows

An **unknown publisher** or SmartScreen reputation warning is distinct from a
Defender malware detection. A matching SHA-256 checksum confirms the archive
matches the release asset; it does not prove that the software is safe.

If Defender reports a threat in the script or executable, stop and report the
exact detection to the maintainers for review. Do not add antivirus exclusions
or disable Defender or SmartScreen. If a managed Windows policy blocks
installation, ask your administrator for approval.

## Uninstall

Remove the installed executable.

### macOS and Linux

```sh
rm "$HOME/.local/bin/runswmmrs"
```

### Windows

```powershell
Remove-Item "$HOME\.local\bin\runswmmrs.exe"
```

If the installer added `~/.local/bin` to PATH, you may also remove that PATH entry from your shell startup file or Windows user environment. If you set `SWMMRS_INSTALL_DIR`, remove `runswmmrs` (or `runswmmrs.exe`) from that custom directory instead.
