param(
  [switch]$SkipInstall
)

$ErrorActionPreference = "Stop"
Set-Location -Path $PSScriptRoot

if (-not (Test-Path "tauri-app")) {
  throw "tauri-app 폴더를 찾을 수 없습니다."
}

Push-Location "tauri-app"
try {
  if (-not $SkipInstall) {
    npm install
  }

  npm run build
}
finally {
  Pop-Location
}
