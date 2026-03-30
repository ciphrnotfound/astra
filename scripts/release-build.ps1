param(
  [string]$Version = ""
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($Version)) {
  $Version = node -e "process.stdout.write(require('./package.json').version)" 2>$null
}

$platform = $env:OS
$arch = $env:PROCESSOR_ARCHITECTURE

$platKey = "win32"
if ($IsLinux) { $platKey = "linux" }
if ($IsMacOS) { $platKey = "darwin" }

$archKey = "x64"
if ($arch -match "ARM64") { $archKey = "arm64" }

cargo build -p astra-cli --release

$ext = "bin"
$src = Join-Path $root "target\release\astra-cli"
if ($IsWindows) {
  $ext = "exe"
  $src = Join-Path $root "target\release\astra-cli.exe"
}

$name = "astra-cli-$Version-$platKey-$archKey.$ext"
$dist = Join-Path $root "dist"
New-Item -ItemType Directory -Force -Path $dist | Out-Null
Copy-Item -Force $src (Join-Path $dist $name)
Write-Output $name
