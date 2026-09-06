# Install TikzForge on Windows.
#
# Downloads the latest release from GitHub and installs it:
#   - NSIS installer .exe → runs the installer silently
#
# Usage:
#   .\install.ps1                 # install latest
#   .\install.ps1 -Version v0.1.0 # install specific version
#   .\install.ps1 -Check          # print latest version, don't install

param(
    [string]$Version = "latest",
    [switch]$Check
)

$ErrorActionPreference = "Stop"
$Repo = "shivamnarkar47/tikzforge"

function Log-Msg { param($msg) Write-Host "[install-tikzforge] $msg" }

function Get-LatestVersion {
    Log-Msg "Fetching latest release..."
    $resp = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    return $resp.tag_name
}

function Get-ReleaseAsset {
    param([string]$ver)

    Log-Msg "Downloading TikzForge $ver..."

    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/tags/$ver"

    # Prefer the NSIS installer.
    $asset = $release.assets | Where-Object { $_.name -like "*.exe" } | Select-Object -First 1

    if (-not $asset) {
        Write-Error "No .exe installer asset found in release $ver."
    }

    Log-Msg "Found asset: $($asset.name)"
    return $asset
}

function Install-Exe {
    param($asset)

    $tmpdir = Join-Path $env:TEMP "tikzforge-install"
    New-Item -ItemType Directory -Force -Path $tmpdir | Out-Null

    $dest = Join-Path $tmpdir $asset.name

    Log-Msg "Downloading to $dest..."
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $dest

    Log-Msg "Running installer (this may take a minute)..."
    # /S = silent install to the default location.
    Start-Process -FilePath $dest -ArgumentList "/S" -Wait

    Log-Msg "Installed. TikzForge should now be in your Start menu."

    # Clean up.
    Remove-Item -Force $dest -ErrorAction SilentlyContinue
}

function Install-AppImage {
    param($asset)

    # Windows users may have AppImage via WSL or AppImageLauncher.
    $installDir = "$env:LOCALAPPDATA\TikzForge"
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null

    $dest = Join-Path $installDir $asset.name

    Log-Msg "Downloading to $dest..."
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $dest

    Log-Msg "Saved to $dest"
    Log-Msg "To run: launch the AppImage directly, or use AppImageLauncher."
}

# Main
Log-Msg "TikzForge installer for Windows"

if ($Version -eq "latest") {
    $Version = Get-LatestVersion
}
Log-Msg "Version: $Version"

if ($Check) {
    Write-Host "Latest version: $Version"
    exit 0
}

$asset = Get-ReleaseAsset -ver $Version

if ($asset.name -like "*.exe") {
    Install-Exe -asset $asset
} else {
    Install-AppImage -asset $asset
}

Log-Msg "Done."
