param(
  [Parameter(Mandatory=$true)]
  [ValidatePattern('^\d+\.\d+\.\d+$')]
  [string]$Version
)
$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
Set-Location $root

# Sync package.json
$pkg=Get-Content '.\package.json' -Raw | ConvertFrom-Json
$pkg.version=$Version
$pkg | ConvertTo-Json -Depth 20 | Set-Content '.\package.json' -Encoding utf8

# Sync Tauri config
$cfg=Get-Content '.\src-tauri\tauri.conf.json' -Raw | ConvertFrom-Json
$cfg.version=$Version
$cfg.app.windows[0].url="index.html?v=$Version"
$cfg | ConvertTo-Json -Depth 30 | Set-Content '.\src-tauri\tauri.conf.json' -Encoding utf8

# Sync Cargo package version
$cargo=Get-Content '.\src-tauri\Cargo.toml' -Raw
$cargo=[regex]::Replace($cargo,'(?ms)(\[package\].*?^version\s*=\s*")([^"]+)(")',"`$1$Version`$3",1)
Set-Content '.\src-tauri\Cargo.toml' $cargo -Encoding utf8

# Sync visible footer version
$index=Get-Content '.\src\index.html' -Raw
$index=[regex]::Replace($index,'Veyra \d+\.\d+(?:\.\d+)? · WebView2',"Veyra $Version · WebView2")
$index=[regex]::Replace($index,'styles\.css\?v=[^"'']+',"styles.css?v=$Version")
$index=[regex]::Replace($index,'/main\.js\?v=[^"'']+',"/main.js?v=$Version")
Set-Content '.\src\index.html' $index -Encoding utf8
$main=Get-Content '.\src\main.js' -Raw
$main=[regex]::Replace($main,'i18n\.js\?v=[^''"]+',"i18n.js?v=$Version")
Set-Content '.\src\main.js' $main -Encoding utf8

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
