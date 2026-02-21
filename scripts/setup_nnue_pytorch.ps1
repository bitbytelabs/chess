$ErrorActionPreference = 'Stop'

$RootDir = Resolve-Path (Join-Path $PSScriptRoot '..')
$NnueDir = Join-Path $RootDir 'external/nnue-pytorch'
$NnueRepo = 'https://github.com/official-stockfish/nnue-pytorch'

if (Test-Path (Join-Path $NnueDir '.git')) {
  Write-Host "nnue-pytorch already present at $NnueDir"
  exit 0
}

New-Item -ItemType Directory -Force -Path (Join-Path $RootDir 'external') | Out-Null
git clone $NnueRepo $NnueDir
Write-Host "Cloned nnue-pytorch into $NnueDir"
