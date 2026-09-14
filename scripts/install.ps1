# Install runswmmrs from a GitHub release.
# Download, review, then run (Windows PowerShell):
# Invoke-WebRequest -UseBasicParsing -Uri "https://raw.githubusercontent.com/swmm-rs/swmmrs/main/scripts/install.ps1" -OutFile "$HOME\Downloads\install-swmmrs.ps1"
# powershell -NoProfile -File "$HOME\Downloads\install-swmmrs.ps1"

$ErrorActionPreference = "Stop"
Set-StrictMode -Version 2.0

$Repository = "swmm-rs/swmmrs"
$Binary = "runswmmrs.exe"
$GitHubBaseUrl = if ($env:SWMMRS_GITHUB_BASE_URL) {
    $env:SWMMRS_GITHUB_BASE_URL.TrimEnd("/")
} else {
    "https://github.com"
}
$GitHubApiUrl = if ($env:SWMMRS_GITHUB_API_URL) {
    $env:SWMMRS_GITHUB_API_URL.TrimEnd("/")
} else {
    "https://api.github.com"
}
$Version = if ($env:SWMMRS_VERSION) { $env:SWMMRS_VERSION } else { "latest" }
$InstallDir = if ($env:SWMMRS_INSTALL_DIR) {
    [IO.Path]::GetFullPath([Environment]::ExpandEnvironmentVariables($env:SWMMRS_INSTALL_DIR))
} else {
    Join-Path $HOME ".local\bin"
}
$NoModifyPath = $env:SWMMRS_NO_MODIFY_PATH -in @("1", "true", "TRUE", "yes", "YES")

function Get-TargetTriple {
    if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
        throw "This installer only supports Windows. Use install.sh on macOS or Linux."
    }

    $nativeArchitecture = if ($env:PROCESSOR_ARCHITEW6432) {
        $env:PROCESSOR_ARCHITEW6432
    } else {
        $env:PROCESSOR_ARCHITECTURE
    }
    if ($nativeArchitecture -eq "ARM64") {
        return "aarch64-pc-windows-msvc"
    }

    try {
        $runtimeAssembly = [System.Reflection.Assembly]::LoadWithPartialName(
            "System.Runtime.InteropServices.RuntimeInformation"
        )
        $runtimeType = $runtimeAssembly.GetType(
            "System.Runtime.InteropServices.RuntimeInformation"
        )
        $architecture = $runtimeType.GetProperty("OSArchitecture").GetValue($null).ToString()
    } catch {
        if ([Environment]::Is64BitOperatingSystem) {
            $architecture = "X64"
        } else {
            throw "Unsupported 32-bit Windows installation."
        }
    }

    switch ($architecture) {
        "X64" { return "x86_64-pc-windows-msvc" }
        "Arm64" { return "aarch64-pc-windows-msvc" }
        default { throw "Unsupported Windows architecture: $architecture" }
    }
}

function Invoke-Download([string] $Url, [string] $Destination) {
    $parameters = @{
        Uri = $Url
        OutFile = $Destination
    }
    if ($PSVersionTable.PSVersion.Major -lt 6) {
        $parameters.UseBasicParsing = $true
    }
    Invoke-WebRequest @parameters
}

function Resolve-LatestRustTag {
    for ($page = 1; $page -le 100; $page++) {
        $parameters = @{
            Uri = "$GitHubApiUrl/repos/$Repository/releases?per_page=100&page=$page"
            Headers = @{ Accept = "application/vnd.github+json" }
        }
        # Invoke-RestMethod can emit a JSON array as one pipeline object.
        # Assign first, then normalize it so Where-Object sees each release.
        $response = Invoke-RestMethod @parameters
        $releases = @($response)
        $release = $releases | Where-Object {
            -not $_.draft -and
            -not $_.prerelease -and
            $_.tag_name -match "^rs-(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$"
        } | Select-Object -First 1
        if ($release) {
            return $release.tag_name
        }
        if ($releases.Count -lt 100) {
            break
        }
    }
    throw "No stable rs-* GitHub release is available."
}

function Add-ToUserPath([string] $Directory) {
    if ($NoModifyPath) {
        return
    }

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $entries = if ($userPath) { $userPath -split ";" } else { @() }
    $normalizedDirectory = $Directory.TrimEnd('\')
    $normalizedEntries = $entries | ForEach-Object { $_.TrimEnd('\') }
    if ($normalizedEntries -notcontains $normalizedDirectory) {
        $newUserPath = (@($Directory) + $entries | Where-Object { $_ }) -join ";"
        [Environment]::SetEnvironmentVariable("Path", $newUserPath, "User")
        Write-Host "Added $Directory to the user PATH"
    }

    $processEntries = $env:Path -split ";"
    if ($processEntries -notcontains $Directory) {
        $env:Path = "$Directory;$env:Path"
    }
}

$tag = if ($Version -eq "latest") {
    Resolve-LatestRustTag
} elseif ($Version.StartsWith("rs-")) {
    $Version
} elseif ($Version.StartsWith("v")) {
    "rs-$($Version.Substring(1))"
} else {
    "rs-$Version"
}
if ($tag -notmatch "^rs-(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:[-.+][0-9A-Za-z.-]+)?$") {
    throw "SWMMRS_VERSION must be latest or a Rust release such as rs-0.1.0."
}

$target = Get-TargetTriple
$asset = "runswmmrs-$target.zip"
$releasePath = "download/$tag"
$displayVersion = $tag
$baseUrl = "$GitHubBaseUrl/$Repository/releases/$releasePath"
$tempDir = Join-Path ([IO.Path]::GetTempPath()) ("swmmrs-" + [Guid]::NewGuid())
$archive = Join-Path $tempDir $asset
$checksumFile = "$archive.sha256"
$unpacked = Join-Path $tempDir "unpacked"
$stagedDestination = $null

try {
    New-Item -ItemType Directory -Path $tempDir | Out-Null
    Write-Host "Downloading runswmmrs $displayVersion for $target"
    Invoke-Download "$baseUrl/$asset" $archive
    Invoke-Download "$baseUrl/$asset.sha256" $checksumFile

    $checksumText = (Get-Content -Path $checksumFile -Raw).Trim()
    if ($checksumText -notmatch "^([0-9A-Fa-f]{64})(?:\s|$)") {
        throw "Release checksum is not valid SHA-256."
    }
    $expectedChecksum = $Matches[1].ToLowerInvariant()
    $actualChecksum = (Get-FileHash -Path $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualChecksum -ne $expectedChecksum) {
        throw "Checksum verification failed for $asset."
    }

    New-Item -ItemType Directory -Path $unpacked | Out-Null
    Expand-Archive -Path $archive -DestinationPath $unpacked
    $sourceBinary = Join-Path $unpacked $Binary
    if (-not (Test-Path -LiteralPath $sourceBinary -PathType Leaf)) {
        throw "$asset does not contain $Binary."
    }

    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    $destination = Join-Path $InstallDir $Binary
    $stagedDestination = "$destination.new.$PID"
    Copy-Item -LiteralPath $sourceBinary -Destination $stagedDestination -Force
    Move-Item -LiteralPath $stagedDestination -Destination $destination -Force
    Add-ToUserPath $InstallDir

    Write-Host "Installed runswmmrs to $destination"
    if ($NoModifyPath) {
        Write-Host "Add $InstallDir to PATH to run runswmmrs from a new terminal."
    } else {
        Write-Host "Open a new terminal, then run: runswmmrs --help"
    }
} finally {
    if ($stagedDestination -and (Test-Path -LiteralPath $stagedDestination)) {
        Remove-Item -LiteralPath $stagedDestination -Force
    }
    if (Test-Path -LiteralPath $tempDir) {
        Remove-Item -LiteralPath $tempDir -Recurse -Force
    }
}
