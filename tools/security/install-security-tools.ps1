# Windows / PowerShell installer for the Aeon security toolchain.
# Mirrors tools/security/install-security-tools.sh — same tool list and
# same version pins from security/toolchain.lock.

param(
  [switch]$Check,
  [switch]$Offline
)

$ErrorActionPreference = "Stop"
$here = Resolve-Path "$PSScriptRoot/../.."
Set-Location $here

$toolDir = Join-Path $here ".aeon-tools\bin"
New-Item -ItemType Directory -Force -Path $toolDir | Out-Null
$env:PATH = "$toolDir;$env:PATH"

function Log($msg) { Write-Host "[sec-tools] $msg" -ForegroundColor Cyan }
function Die($msg) { Write-Host "[sec-tools] $msg" -ForegroundColor Red; exit 1 }

function Install-Cargo-Tool($name, $version) {
  if ($Offline) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) { Die "$name absent + -Offline set" }
    return
  }
  cargo install --locked --version $version $name --root "$here\.aeon-tools"
}

function Install-Gitleaks($version) {
  if (Get-Command gitleaks -ErrorAction SilentlyContinue) { return }
  if ($Offline) { Die "gitleaks absent + -Offline set" }
  $arch = if ([Environment]::Is64BitOperatingSystem) { "x64" } else { Die "gitleaks requires 64-bit windows" }
  $url = "https://github.com/gitleaks/gitleaks/releases/download/v${version}/gitleaks_${version}_windows_${arch}.zip"
  $tmp = New-TemporaryFile
  Invoke-WebRequest -Uri $url -OutFile $tmp
  Expand-Archive -Path $tmp -DestinationPath $toolDir -Force
}

if (-not $Check) {
  Install-Cargo-Tool "cargo-audit"     "0.21.1"
  Install-Cargo-Tool "cargo-cyclonedx" "0.5.7"
  Install-Gitleaks "8.28.0"
  rustup component add rustfmt
  rustup component add clippy
}

Log "installed security tools in $toolDir"
