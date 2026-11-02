param(
    [switch]$AcceptLicense
)

$ErrorActionPreference = "Stop"

if (-not $AcceptLicense) {
    Write-Host "This script downloads the Live2D sample model 'Tororo & Hijiki' and the Cubism Core runtime."
    Write-Host ""
    Write-Host "Before continuing you must agree to:"
    Write-Host "  - Live2D Free Material License Agreement"
    Write-Host "  - Live2D Cubism Sample Data Terms of Use"
    Write-Host "  - Live2D Proprietary Software License Agreement (for Cubism Core)"
    Write-Host ""
    Write-Host "Official terms:"
    Write-Host "  https://www.live2d.com/eula/live2d-free-material-license-agreement_en.html"
    Write-Host "  https://www.live2d.com/learn/sample/model-terms/"
    Write-Host ""
    Write-Host "Re-run with -AcceptLicense after reading the terms."
    exit 1
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$publicLive2D = Join-Path $repoRoot "public/live2d"
$modelDir = Join-Path $publicLive2D "tororo"
$coreDir = Join-Path $publicLive2D "core"
$zipPath = Join-Path $env:TEMP "tororo_hijiki_ja.zip"
$extractDir = Join-Path $env:TEMP "moyuyu-tororo-extract"

New-Item -ItemType Directory -Force -Path $modelDir, $coreDir | Out-Null

$modelUrl = "https://cubism.live2d.com/sample-data/bin/tororo_hijiki/tororo_hijiki_ja.zip"
$coreUrl = "https://cubism.live2d.com/sdk-web/cubismcore/live2dcubismcore.min.js"

Write-Host "Downloading Tororo model..."
Invoke-WebRequest -Uri $modelUrl -OutFile $zipPath -UseBasicParsing

Write-Host "Extracting runtime files..."
Remove-Item $extractDir -Recurse -Force -ErrorAction SilentlyContinue
Expand-Archive -Path $zipPath -DestinationPath $extractDir -Force
Copy-Item (Join-Path $extractDir "tororo/runtime/*") $modelDir -Recurse -Force

$readme = Join-Path $extractDir "ReadMe.txt"
if (Test-Path $readme) {
    Copy-Item $readme (Join-Path $modelDir "LICENSE.txt") -Force
}

Write-Host "Downloading Cubism Core..."
Invoke-WebRequest -Uri $coreUrl -OutFile (Join-Path $coreDir "live2dcubismcore.min.js") -UseBasicParsing

Write-Host ""
Write-Host "Live2D assets installed:"
Write-Host "  $modelDir"
Write-Host "  $coreDir\\live2dcubismcore.min.js"
Write-Host ""
Write-Host "Attribution: Tororo & Hijiki sample model by Live2D Inc., used under the Live2D Free Material License."
