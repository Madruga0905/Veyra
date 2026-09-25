param(
  [Parameter(Mandatory=$true)]
  [ValidatePattern('^\d+\.\d+\.\d+$')]
  [string]$Version
)
$ErrorActionPreference='Stop'
$Utf8NoBom=New-Object System.Text.UTF8Encoding($false)
function Write-Utf8NoBom([string]$Path,[string]$Content){[IO.File]::WriteAllText($Path,$Content,$Utf8NoBom)}
$root=Split-Path -Parent $PSScriptRoot
Set-Location $root

# Sync package.json
$pkg=Get-Content '.\package.json' -Raw | ConvertFrom-Json
$pkg.version=$Version
Write-Utf8NoBom '.\package.json' ($pkg | ConvertTo-Json -Depth 20)

# Sync Tauri config
$cfg=Get-Content '.\src-tauri\tauri.conf.json' -Raw | ConvertFrom-Json
$cfg.version=$Version
$cfg.app.windows[0].url="index.html?v=$Version"
Write-Utf8NoBom '.\src-tauri\tauri.conf.json' ($cfg | ConvertTo-Json -Depth 30)

# Sync Cargo package version
$cargo=Get-Content '.\src-tauri\Cargo.toml' -Raw
$cargo=[regex]::Replace($cargo,'(?m)^version\s*=\s*"[^"]+"',('version = "'+$Version+'"'),1)
Write-Utf8NoBom '.\src-tauri\Cargo.toml' $cargo

# Sync visible footer version
$index=Get-Content '.\src\index.html' -Raw
$index=[regex]::Replace($index,'Veyra \d+\.\d+(?:\.\d+)? [^<]*?WebView2',"Veyra $Version · WebView2")
$index=[regex]::Replace($index,'styles\.css\?v=[^"'']+',"styles.css?v=$Version")
$index=[regex]::Replace($index,'/main\.js\?v=[^"'']+',"/main.js?v=$Version")
Write-Utf8NoBom '.\src\index.html' $index
$main=Get-Content '.\src\main.js' -Raw
$main=[regex]::Replace($main,'i18n\.js\?v=[^''"]+',"i18n.js?v=$Version")
Write-Utf8NoBom '.\src\main.js' $main

# Refresh lockfile metadata without changing dependencies
npm install --package-lock-only --ignore-scripts | Out-Host

node --check '.\src\main.js'
node --check '.\src\i18n.js'
cargo check --manifest-path '.\src-tauri\Cargo.toml'
if($LASTEXITCODE -ne 0){throw 'Validation failed'}

Write-Host "Veyra $Version is ready for release." -ForegroundColor Green
Write-Host "Commit the changes, then push tag v$Version to trigger GitHub Actions."
Write-Host "  git add ."
Write-Host "  git commit -m 'Release Veyra v$Version'"
Write-Host "  git tag v$Version"
Write-Host "  git push origin main --tags"
