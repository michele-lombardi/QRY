param(
    [ValidateSet("x86_64-pc-windows-msvc")]
    [string]$Target = "x86_64-pc-windows-msvc"
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$ScriptDirectory = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = (Resolve-Path (Join-Path $ScriptDirectory "..")).Path
$AppRoot = Join-Path $ProjectRoot "QRY"
$ReleaseDirectory = Join-Path $ProjectRoot "release"
$Package = Get-Content (Join-Path $AppRoot "package.json") -Raw | ConvertFrom-Json
$TauriConfig = Get-Content (Join-Path $AppRoot "src-tauri/tauri.conf.json") -Raw | ConvertFrom-Json
$WorkspaceManifest = Get-Content (Join-Path $AppRoot "Cargo.toml") -Raw
$CargoVersionMatch = [regex]::Match(
    $WorkspaceManifest,
    '(?ms)^\[workspace\.package\].*?^version\s*=\s*"(?<version>[^"]+)"'
)
if (-not $CargoVersionMatch.Success) {
    throw "Cannot read workspace version from QRY/Cargo.toml"
}

$Version = [string]$Package.version
$MetadataVersions = @($Version, [string]$TauriConfig.version, $CargoVersionMatch.Groups["version"].Value)
if (@($MetadataVersions | Select-Object -Unique).Count -ne 1) {
    throw "Version mismatch across package.json, tauri.conf.json and Cargo.toml: $($MetadataVersions -join ', ')"
}
if ([string]$TauriConfig.productName -ne "QRY") {
    throw "Unexpected Windows product name: $($TauriConfig.productName)"
}

$BundleRoot = Join-Path $AppRoot "target/$Target/release/bundle"
$NsisDirectory = Join-Path $BundleRoot "nsis"
$MsiDirectory = Join-Path $BundleRoot "msi"
$NsisFiles = @(Get-ChildItem -Path $NsisDirectory -File -Filter "*-setup.exe")
$MsiFiles = @(Get-ChildItem -Path $MsiDirectory -File -Filter "*.msi")
$ApplicationExecutable = Join-Path $AppRoot "target/$Target/release/typepulse-app.exe"
if ($NsisFiles.Count -ne 1) {
    throw "Expected exactly one NSIS installer in $NsisDirectory; found $($NsisFiles.Count)"
}
if ($MsiFiles.Count -ne 1) {
    throw "Expected exactly one MSI installer in $MsiDirectory; found $($MsiFiles.Count)"
}
if (-not (Test-Path -LiteralPath $ApplicationExecutable -PathType Leaf)) {
    throw "Expected release executable at $ApplicationExecutable"
}

function Assert-ValidAuthenticodeSignature([string]$Path) {
    $Signature = Get-AuthenticodeSignature -FilePath $Path
    if ($Signature.Status -ne "Valid") {
        throw "Authenticode signature is not valid for $Path`: $($Signature.Status)"
    }
}

if ($env:QRY_REQUIRE_WINDOWS_SIGNATURE -eq "1") {
    Assert-ValidAuthenticodeSignature $ApplicationExecutable
}

& node (Join-Path $ScriptDirectory "audit-release-content.mjs") $NsisDirectory $MsiDirectory
if ($LASTEXITCODE -ne 0) {
    throw "Windows bundle content audit failed"
}

New-Item -ItemType Directory -Force -Path $ReleaseDirectory | Out-Null
$Assets = @(
    @{
        Source = $NsisFiles[0].FullName
        Name = "QRY_${Version}_x64-setup.exe"
    },
    @{
        Source = $MsiFiles[0].FullName
        Name = "QRY_${Version}_x64_en-US.msi"
    }
)

foreach ($Asset in $Assets) {
    $Destination = Join-Path $ReleaseDirectory $Asset.Name
    Copy-Item -LiteralPath $Asset.Source -Destination $Destination -Force

    if ($env:QRY_REQUIRE_WINDOWS_SIGNATURE -eq "1") {
        Assert-ValidAuthenticodeSignature $Destination
    }

    $Hash = (Get-FileHash -LiteralPath $Destination -Algorithm SHA256).Hash.ToLowerInvariant()
    Set-Content -LiteralPath "$Destination.sha256" -Value "$Hash  $($Asset.Name)" -Encoding ascii
    Write-Host "Created $Destination and $Destination.sha256"
}
