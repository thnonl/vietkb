# Builds the Microsoft Store (MSIX) package.
#
#   scripts\build-store.ps1                      unsigned .msix (what Partner Center takes)
#   scripts\build-store.ps1 -Arch arm64          same for Windows on Arm (x64, x86, arm64)
#   scripts\build-store.ps1 -Sign                same, signed with a local self-signed cert
#   scripts\build-store.ps1 -Sign -CertPath x.pfx -CertPassword (Read-Host -AsSecureString)
#
# The exe is the normal release build plus the `store` cargo feature: autostart
# through the manifest's StartupTask, no self-updater, no network code. Same tauri
# invocation as the NSIS build (build-prod.ps1, release.yml) except `--no-bundle`
# and the feature. Like build-dev.ps1 it uses its OWN cargo target dir
# (`target-store`), so a store exe never lands in `target\release` where the NSIS
# build (build-prod.ps1, release.yml) takes its exe from.
#
# Output: target-store\store\VietKB_<version>_<arch>.msix (staging dir: target-store\store\stage).
# x86 and arm64 cross-compile (`--target`); arm64 needs the MSVC ARM64 build tools.
# Partner Center takes one .msix per architecture in the same submission.
#
# Store submission is UNSIGNED: the Store signs the package itself. -Sign is only
# for installing the package locally (Add-AppxPackage) to test it. A self-signed
# package installs only after its certificate is trusted (the script says how).

[CmdletBinding()]
param(
    [ValidateSet('x64', 'x86', 'arm64')]
    [string]$Arch = 'x64',
    [switch]$Sign,
    [string]$CertPath,
    [securestring]$CertPassword,
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$tauri = Join-Path $root 'ui\node_modules\.bin\tauri.cmd'
$packaging = Join-Path $root 'packaging\msix'
$targetDir = Join-Path $root 'target-store'
$outDir = Join-Path $targetDir 'store'
$stage = Join-Path $outDir 'stage'
$exeName = 'vietkb.exe'
# x64 builds for the host as before (exe in target-store\release); the others
# cross-compile into target-store\<triple>\release.
$triple = @{ x64 = $null; x86 = 'i686-pc-windows-msvc'; arm64 = 'aarch64-pc-windows-msvc' }[$Arch]

# Same publisher as AppxManifest.xml; a signing cert must carry this exact subject.
$manifestTemplate = Get-Content (Join-Path $packaging 'AppxManifest.xml') -Raw -Encoding UTF8
if ($manifestTemplate -notmatch 'Publisher="([^"]+)"') { throw 'no Publisher in AppxManifest.xml' }
$publisher = $Matches[1]
if ($manifestTemplate -notmatch 'Application Id="App" Executable="([^"]+)"') { throw 'no Application Executable in AppxManifest.xml' }
if ($Matches[1] -ne $exeName) { throw "manifest Executable '$($Matches[1])' is not '$exeName'" }

# Version: tauri.conf.json is the release version (release.yml reads it too); the
# workspace Cargo version is a placeholder. MSIX wants four parts; the Store
# reserves a revision of 0, so 0.2.7 becomes 0.2.7.0.
$conf = Get-Content (Join-Path $root 'crates\vkb-app\tauri.conf.json') -Raw -Encoding UTF8 | ConvertFrom-Json
if ($conf.version -notmatch '^(\d+)\.(\d+)\.(\d+)') { throw "unexpected version '$($conf.version)'" }
$version = "$($Matches[1]).$($Matches[2]).$($Matches[3]).0"
Write-Host "Version: $version"

# --- build ---------------------------------------------------------------------
if (-not $SkipBuild) {
    if (-not (Test-Path $tauri)) {
        Write-Host 'Installing UI dependencies (npm ci)...'
        & npm.cmd --prefix (Join-Path $root 'ui') ci
        if ($LASTEXITCODE -ne 0) { throw "npm ci failed ($LASTEXITCODE)" }
    }
    # `tauri build` runs the UI build (beforeBuildCommand) and adds custom-protocol
    # itself; it is spelled out as in build-prod.ps1.
    # `tauri build` has no `--target-dir`; cargo reads CARGO_TARGET_DIR.
    $previousTargetDir = $env:CARGO_TARGET_DIR
    $env:CARGO_TARGET_DIR = $targetDir
    Push-Location (Join-Path $root 'crates\vkb-app')
    try {
        $targetArgs = if ($triple) { @('--target', $triple) } else { @() }
        & $tauri build --features custom-protocol,store --no-bundle @targetArgs
        if ($LASTEXITCODE -ne 0) { throw "tauri build failed ($LASTEXITCODE)" }
    } finally {
        Pop-Location
        $env:CARGO_TARGET_DIR = $previousTargetDir
    }
}

$exe = if ($triple) { Join-Path $targetDir "$triple\release\$exeName" } else { Join-Path $targetDir "release\$exeName" }
if (-not (Test-Path $exe)) { throw "built exe not found: $exe" }

# --- stage ---------------------------------------------------------------------
if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
New-Item -ItemType Directory -Path $stage | Out-Null
Copy-Item $exe $stage
# Licence notices of the embedded data (same file the NSIS/macOS bundles carry as a resource).
Copy-Item (Join-Path $root 'THIRD_PARTY_NOTICES.md') $stage
Copy-Item (Join-Path $packaging 'Assets') (Join-Path $stage 'Assets') -Recurse
$manifest = $manifestTemplate.Replace('__VERSION__', $version).Replace('__ARCH__', $Arch)
# UTF-8 without BOM, as makeappx expects.
[System.IO.File]::WriteAllText((Join-Path $stage 'AppxManifest.xml'), $manifest, (New-Object System.Text.UTF8Encoding($false)))

# --- makeappx ------------------------------------------------------------------
function Find-SdkTool([string]$name) {
    $kits = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
    $found = Get-ChildItem $kits -Directory -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -match '^\d+\.\d+\.\d+\.\d+$' } |
        Sort-Object { [version]$_.Name } -Descending |
        ForEach-Object { Join-Path $_.FullName "x64\$name" } |
        Where-Object { Test-Path $_ } |
        Select-Object -First 1
    if (-not $found) { throw "$name not found under $kits (install the Windows 10/11 SDK)" }
    return $found
}

$msix = Join-Path $outDir "VietKB_${version}_${Arch}.msix"
if (Test-Path $msix) { Remove-Item $msix -Force }
$makeappx = Find-SdkTool 'makeappx.exe'
Write-Host "makeappx: $makeappx"
& $makeappx pack /d $stage /p $msix /o
if ($LASTEXITCODE -ne 0) { throw "makeappx failed ($LASTEXITCODE)" }

# --- optional local signing ----------------------------------------------------
if ($Sign) {
    $signtool = Find-SdkTool 'signtool.exe'
    if (-not $CertPath) {
        $CertPath = Join-Path $outDir 'VietKB-test.pfx'
        $cerPath = Join-Path $outDir 'VietKB-test.cer'
        if (-not $CertPassword) { $CertPassword = ConvertTo-SecureString 'vietkb-test' -AsPlainText -Force }
        $cert = New-SelfSignedCertificate -Type Custom -Subject $publisher -KeyUsage DigitalSignature `
            -FriendlyName 'VietKB local test' -CertStoreLocation 'Cert:\CurrentUser\My' `
            -TextExtension @('2.5.29.37={text}1.3.6.1.5.5.7.3.3', '2.5.29.19={text}')
        Export-PfxCertificate -Cert $cert -FilePath $CertPath -Password $CertPassword | Out-Null
        Export-Certificate -Cert $cert -FilePath $cerPath | Out-Null
        Write-Host "Self-signed certificate: $CertPath (password 'vietkb-test' unless -CertPassword was given)"
        Write-Host "To install the package, trust it once (elevated): Import-Certificate -FilePath '$cerPath' -CertStoreLocation Cert:\LocalMachine\TrustedPeople"
    }
    if (-not $CertPassword) { throw '-CertPath needs -CertPassword' }
    $plain = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto([System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($CertPassword))
    & $signtool sign /fd SHA256 /f $CertPath /p $plain $msix
    if ($LASTEXITCODE -ne 0) { throw "signtool failed ($LASTEXITCODE)" }
}

$info = Get-Item $msix
Write-Host ''
Write-Host "Package: $($info.FullName)  ($([math]::Round($info.Length / 1MB, 2)) MB)"
if ($Sign) {
    Write-Host "Install: Add-AppxPackage '$($info.FullName)'"
} else {
    Write-Host 'Unsigned: upload to Partner Center (the Store signs it). To test locally, rerun with -Sign.'
}
