Param(
  [string]$Version = "",
  [string]$OutDir = "dist"
)

$ErrorActionPreference = "Stop"

Write-Host "Building release binary..."
cargo build --release

$binName = "lerobot-servo-adjust"
$exe = Join-Path -Path "target/release" -ChildPath "$binName.exe"
if (-not (Test-Path $exe)) {
  # non-windows build might not have .exe
  $exe = Join-Path -Path "target/release" -ChildPath $binName
}

if ([string]::IsNullOrEmpty($Version)) {
  $toml = Get-Content Cargo.toml -Raw
  if ($toml -match 'version\s*=\s*"([^"]+)"') { $Version = $Matches[1] } else { $Version = "0.0.0" }
}

$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLower()
$platform = if ($IsWindows) { "windows" } elseif ($IsLinux) { "linux" } elseif ($IsMacOS) { "macos" } else { "unknown" }

$bundle = "${binName}-${Version}-${platform}-${arch}"
$root = Join-Path -Path $OutDir -ChildPath $bundle
if (Test-Path $root) { Remove-Item -Recurse -Force $root }
New-Item -ItemType Directory -Force -Path $root | Out-Null

# layout
$binDir = Join-Path $root "bin"
$tplDir = Join-Path $root "templates"
$dataDir = Join-Path $root "huggingface"
New-Item -ItemType Directory -Force -Path $binDir,$tplDir | Out-Null

Copy-Item $exe -Destination (Join-Path $binDir ([IO.Path]::GetFileName($exe))) -Force
Copy-Item -Recurse templates\* $tplDir -Force
if (Test-Path "huggingface") { Copy-Item -Recurse "huggingface" $root -Force }

Copy-Item README.md,DEVELOP.md,GUIDE.md -Destination $root -Force -ErrorAction SilentlyContinue

# zip
Write-Host "Creating archive..."
if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Force -Path $OutDir | Out-Null }
$zipPath = Join-Path -Path $OutDir -ChildPath ("$bundle.zip")
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Compress-Archive -Path (Join-Path $root '*') -DestinationPath $zipPath

Write-Host "Packed:" $zipPath

