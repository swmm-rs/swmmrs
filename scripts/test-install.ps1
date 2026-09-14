# Test the PowerShell installer entirely against a local, mocked release server.

$ErrorActionPreference = "Stop"
Set-StrictMode -Version 2.0

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Root = Split-Path -Parent $ScriptDir
$Installer = Join-Path $ScriptDir "install.ps1"
$TempDir = Join-Path ([IO.Path]::GetTempPath()) ("swmmrs-install-test-" + [Guid]::NewGuid())
$Server = $null
$OldEnvironment = @{}

function Require-Command([string] $Name) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "test-install.ps1: required command not found: $Name"
    }
}

try {
    Require-Command "Compress-Archive"
    $pythonCommand = Get-Command python -CommandType Application -ErrorAction SilentlyContinue
    if (-not $pythonCommand) {
        throw "test-install.ps1: a direct python executable is required for the local release server; the py launcher is not supported"
    }
    $pythonExecutable = $pythonCommand.Source
    if (-not $pythonExecutable -or -not (Test-Path -LiteralPath $pythonExecutable -PathType Leaf)) {
        throw "test-install.ps1: python did not resolve to a direct executable"
    }

    $tag = "rs-9.9.9"
    $targets = @("x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc")
    $releaseDir = Join-Path $TempDir "swmm-rs\swmmrs\releases\download\$tag"
    $apiDir = Join-Path $TempDir "api\repos\swmm-rs\swmmrs"
    $installDir = Join-Path $TempDir "install"
    $portFile = Join-Path $TempDir "server.port"
    $serverScript = Join-Path $TempDir "server.py"
    $mockBinary = Join-Path $TempDir "runswmmrs.exe"

    New-Item -ItemType Directory -Path $releaseDir, $apiDir -Force | Out-Null
    @'
[{"draft":false,"prerelease":false,"tag_name":"py-9.9.10"},{"prerelease":true,"tag_name":"rs-10.0.0-beta.1","draft":false},{"prerelease":false,"tag_name":"rs-9.9.9","draft":false}]
'@ | Set-Content -Path (Join-Path $apiDir "releases") -Encoding ascii
    @'
param([string] $Command)
if ($Command -eq "--version") {
    Write-Output "runswmmrs mock 9.9.9"
} else {
    Write-Output "runswmmrs mock executable"
}
'@ | Set-Content -Path $mockBinary -Encoding ascii
    foreach ($target in $targets) {
        $asset = "runswmmrs-$target.zip"
        $archive = Join-Path $releaseDir $asset
        Compress-Archive -Path $mockBinary -DestinationPath $archive
        $hash = (Get-FileHash -Path $archive -Algorithm SHA256).Hash.ToLowerInvariant()
        "$hash  $asset" | Set-Content -Path "$archive.sha256" -Encoding ascii
    }

    @'
import http.server
import pathlib
import socketserver
import sys

root = pathlib.Path(sys.argv[1]).resolve()
port_file = pathlib.Path(sys.argv[2])
handler = lambda *args, **kwargs: http.server.SimpleHTTPRequestHandler(
    *args, directory=str(root), **kwargs
)
with socketserver.TCPServer(("127.0.0.1", 0), handler) as server:
    port_file.write_text(str(server.server_address[1]), encoding="ascii")
    server.serve_forever()
'@ | Set-Content -Path $serverScript -Encoding ascii

    $arguments = "-u `"$serverScript`" `"$TempDir`" `"$portFile`""
    $Server = Start-Process -FilePath $pythonExecutable -ArgumentList $arguments -PassThru -WindowStyle Hidden
    for ($attempt = 0; $attempt -lt 100; $attempt++) {
        if ($Server.HasExited) {
            throw "test-install.ps1: local release server exited early with code $($Server.ExitCode)"
        }
        if ((Test-Path -LiteralPath $portFile) -and
            -not [string]::IsNullOrWhiteSpace((Get-Content -LiteralPath $portFile -Raw))) {
            break
        }
        Start-Sleep -Milliseconds 50
    }
    if ($Server.HasExited) {
        throw "test-install.ps1: local release server exited early with code $($Server.ExitCode)"
    }
    if (-not (Test-Path -LiteralPath $portFile) -or
        [string]::IsNullOrWhiteSpace((Get-Content -LiteralPath $portFile -Raw))) {
        throw "test-install.ps1: local release server did not write a nonempty port"
    }
    $port = (Get-Content -LiteralPath $portFile -Raw).Trim()

    foreach ($name in @(
        "SWMMRS_GITHUB_BASE_URL",
        "SWMMRS_GITHUB_API_URL",
        "SWMMRS_VERSION",
        "SWMMRS_INSTALL_DIR",
        "SWMMRS_NO_MODIFY_PATH"
    )) {
        $OldEnvironment[$name] = [Environment]::GetEnvironmentVariable($name, "Process")
    }
    $oldPath = $env:Path
    $env:SWMMRS_GITHUB_BASE_URL = "http://127.0.0.1:$port"
    $env:SWMMRS_GITHUB_API_URL = "http://127.0.0.1:$port/api"
    $env:SWMMRS_VERSION = "latest"
    $env:SWMMRS_INSTALL_DIR = $installDir
    $env:SWMMRS_NO_MODIFY_PATH = "1"

    & $Installer

    $installed = Join-Path $installDir "runswmmrs.exe"
    if (-not (Test-Path -LiteralPath $installed -PathType Leaf)) {
        throw "installer did not create the executable"
    }
    $contents = Get-Content -LiteralPath $installed -Raw
    if ($contents -notmatch "runswmmrs mock 9\.9\.9") {
        throw "installed executable did not retain the expected mock contents"
    }
    if ($env:Path -ne $oldPath) {
        throw "SWMMRS_NO_MODIFY_PATH changed the process PATH"
    }

    # The mock is a PowerShell payload stored under the release executable
    # name. Execute that payload as a script block so the test verifies
    # behavior without requiring a compiler or a real Windows PE binary.
    $scriptBlock = [scriptblock]::Create($contents)
    $behavior = & $scriptBlock --version
    if (($behavior -join "`n").Trim() -ne "runswmmrs mock 9.9.9") {
        throw "installed mock executable did not produce the expected behavior"
    }

    $legacyInstallDir = Join-Path $TempDir "install-legacy-version"
    $env:SWMMRS_VERSION = "v9.9.9"
    $env:SWMMRS_INSTALL_DIR = $legacyInstallDir
    & $Installer
    $legacyInstalled = Join-Path $legacyInstallDir "runswmmrs.exe"
    if (-not (Test-Path -LiteralPath $legacyInstalled -PathType Leaf)) {
        throw "legacy v-prefixed version did not resolve to the rs-* release"
    }

    Write-Output "PowerShell installer mock test passed"
} finally {
    foreach ($name in $OldEnvironment.Keys) {
        [Environment]::SetEnvironmentVariable($name, $OldEnvironment[$name], "Process")
    }
    if ($Server) {
        Stop-Process -Id $Server.Id -Force -ErrorAction SilentlyContinue
        $Server.Dispose()
    }
    if (Test-Path -LiteralPath $TempDir) {
        Remove-Item -LiteralPath $TempDir -Recurse -Force
    }
}
