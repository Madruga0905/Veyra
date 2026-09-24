param([string]$Version='0.4.0')
$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
Set-Location $root
$nsis=".\release\Veyra_${Version}_x64-setup.exe"
$msi=".\release\Veyra_${Version}_x64_en-US.msi"
$key='.\.signing\veyra.key'
foreach($f in @($nsis,$msi,$key)){if(!(Test-Path $f)){throw "Missing: $f"}}
$secure=Read-Host 'Veyra updater key password' -AsSecureString
$ptr=[Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
try{
  $plain=[Runtime.InteropServices.Marshal]::PtrToStringBSTR($ptr)
  $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD=$plain
  & '.\node_modules\.bin\tauri.cmd' signer sign -f $key --app-version $Version $nsis
  if($LASTEXITCODE){throw 'NSIS signing failed'}
  & '.\node_modules\.bin\tauri.cmd' signer sign -f $key --app-version $Version $msi
  if($LASTEXITCODE){throw 'MSI signing failed'}
}finally{
  $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD=$null
  if($ptr -ne [IntPtr]::Zero){[Runtime.InteropServices.Marshal]::ZeroFreeBSTR($ptr)}
}
$sig=(Get-Content "$nsis.sig" -Raw).Trim()
$asset="Veyra_${Version}_x64-setup.exe"
$url="https://github.com/Madruga0905/Veyra/releases/download/v$Version/$asset"
$latest=[ordered]@{
  version=$Version
  notes="Veyra $Version for Windows"
  pub_date=(Get-Date).ToUniversalTime().ToString('o')
  platforms=@{'windows-x86_64'=@{signature=$sig;url=$url}}
}
$latest | ConvertTo-Json -Depth 8 | Set-Content '.\release\latest.json' -Encoding utf8
$input="protocol=https`nhost=github.com`n`n"
$cred=$input | git credential fill
$user=(($cred|Where-Object{$_ -like 'username=*'}) -replace '^username=','')
$token=(($cred|Where-Object{$_ -like 'password=*'}) -replace '^password=','')
if(!$token){throw 'GitHub credential not found'}
$headers=@{Authorization="Bearer $token";Accept='application/vnd.github+json';'X-GitHub-Api-Version'='2022-11-28'}
$repo='Madruga0905/Veyra'
try{$null=Invoke-RestMethod -Headers $headers -Uri "https://api.github.com/repos/$repo"}
catch{
  if($_.Exception.Response.StatusCode.value__ -ne 404){throw}
  $body=@{name='Veyra';description='Official Veyra browser releases and updater feed';private=$false;auto_init=$true}|ConvertTo-Json
  $null=Invoke-RestMethod -Method Post -Headers $headers -ContentType 'application/json' -Body $body -Uri 'https://api.github.com/user/repos'
  Start-Sleep -Seconds 2
}
$tag="v$Version"
try{$release=Invoke-RestMethod -Headers $headers -Uri "https://api.github.com/repos/$repo/releases/tags/$tag"}
catch{
  if($_.Exception.Response.StatusCode.value__ -ne 404){throw}
  $body=@{tag_name=$tag;name="Veyra $tag";body="Veyra $Version for Windows. Includes NSIS/MSI installers and signed updater metadata.";draft=$false;prerelease=$false}|ConvertTo-Json
  $release=Invoke-RestMethod -Method Post -Headers $headers -ContentType 'application/json' -Body $body -Uri "https://api.github.com/repos/$repo/releases"
}
foreach($a in @($release.assets)){
  if($a.name -in @((Split-Path $nsis -Leaf),(Split-Path $msi -Leaf),((Split-Path $nsis -Leaf)+'.sig'),((Split-Path $msi -Leaf)+'.sig'),'latest.json')){
    Invoke-RestMethod -Method Delete -Headers $headers -Uri "https://api.github.com/repos/$repo/releases/assets/$($a.id)" | Out-Null
  }
}
$uploadBase=($release.upload_url -replace '\{\?name,label\}$','')
$files=@($nsis,"$nsis.sig",$msi,"$msi.sig",'.\release\latest.json')
foreach($f in $files){
  $name=[uri]::EscapeDataString((Split-Path $f -Leaf))
  Invoke-WebRequest -UseBasicParsing -Method Post -Headers $headers -ContentType 'application/octet-stream' -InFile $f -Uri "$uploadBase?name=$name" | Out-Null
  Write-Host "Uploaded $(Split-Path $f -Leaf)"
}
$token=$null;$plain=$null;$secure=$null
Write-Host "Veyra $Version release published: https://github.com/$repo/releases/tag/$tag" -ForegroundColor Green
