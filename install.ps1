# Install TikzForge on Windows.
#
# Downloads a release zip, installs it under %LOCALAPPDATA%\TikzForge, and
# creates a Start Menu shortcut. The zip includes the bundled Tectonic engine
# in tectonic\tectonic.exe.
#
# Usage:
#   .\install.ps1                 # install latest
#   .\install.ps1 -Version v0.4.0 # install specific version
#   .\install.ps1 -Check          # print latest version, don't install

param(
    [string]$Version = "latest",
    [switch]$Check
)

$ErrorActionPreference = "Stop"
$Repo = "shivamnarkar47/tikzforge"
$InstallDir = Join-Path $env:LOCALAPPDATA "TikzForge"
$ExeName = "tikzforge-gpui.exe"

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
    $asset = $release.assets |
        Where-Object { $_.name -like "tikzforge-gpui-*-windows-x86_64.zip" } |
        Select-Object -First 1

    if (-not $asset) {
        throw "No Windows x86_64 archive found in release $ver."
    }

    Log-Msg "Found asset: $($asset.name)"
    return $asset
}

function New-StartMenuShortcut {
    param([string]$Target)

    $startMenu = [Environment]::GetFolderPath("Programs")
    $shortcutPath = Join-Path $startMenu "TikzForge.lnk"
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($shortcutPath)
    $shortcut.TargetPath = $Target
    $shortcut.WorkingDirectory = $InstallDir
    $icon = Join-Path $InstallDir "tikzforge.ico"
    if (-not (Test-Path $icon)) {
        try {
            Invoke-WebRequest -Uri "https://raw.githubusercontent.com/$Repo/$Version/legacy/src-tauri/icons/icon.ico" -OutFile $icon -UseBasicParsing
        } catch {
            Remove-Item -Force $icon -ErrorAction SilentlyContinue
        }
    }
    if (Test-Path $icon) {
        $shortcut.IconLocation = "$icon,0"
    } else {
        $shortcut.IconLocation = "$Target,0"
    }
    $shortcut.Save()
    return $shortcutPath
}

function Install-Release {
    param($asset)

    $tmpDir = Join-Path $env:TEMP ("tikzforge-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

    try {
        $zip = Join-Path $tmpDir $asset.name
        Log-Msg "Downloading to $zip..."
        Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zip -UseBasicParsing

        if (Test-Path $InstallDir) {
            Remove-Item -Recurse -Force $InstallDir
        }
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        Expand-Archive -Path $zip -DestinationPath $InstallDir -Force

        $exe = Join-Path $InstallDir $ExeName
        $engine = Join-Path $InstallDir "tectonic\tectonic.exe"
        if (-not (Test-Path $exe)) { throw "Archive does not contain $ExeName." }
        if (-not (Test-Path $engine)) { throw "Archive does not contain the bundled Tectonic engine." }

        $shortcut = New-StartMenuShortcut -Target $exe
        Log-Msg "Installed to $InstallDir"
        Log-Msg "Start Menu shortcut: $shortcut"
        Log-Msg "Uninstall: remove $InstallDir and the TikzForge shortcut."
    }
    finally {
        Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
    }
}

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
Install-Release -asset $asset
Log-Msg "Done."
