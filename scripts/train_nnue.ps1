$ErrorActionPreference = 'Stop'

$RootDir = Resolve-Path (Join-Path $PSScriptRoot '..')
$NnueDir = Join-Path $RootDir 'external/nnue-pytorch'
$VenvDir = Join-Path $NnueDir '.venv'

if (-not (Test-Path $NnueDir)) {
  Write-Error "Missing $NnueDir. Run ./scripts/setup_nnue_pytorch.ps1 first."
  exit 1
}

python -m venv $VenvDir
$Python = Join-Path $VenvDir 'Scripts/python.exe'
if (-not (Test-Path $Python)) {
  $Python = Join-Path $VenvDir 'Scripts/python'
}

& $Python -m pip install --upgrade pip
& $Python -m pip install -r (Join-Path $NnueDir 'requirements.txt')

Write-Host @"

nnue-pytorch is installed and ready.

Next, run training using the upstream scripts, for example:
  cd external/nnue-pytorch
  python train.py --help

After you produce a network, integrate inference in this engine before expecting Elo gains.
"@
