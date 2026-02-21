$ErrorActionPreference = 'Stop'

$versionFile = 'VERSION'
$cargoFile = 'Cargo.toml'
$changelogFile = 'CHANGELOG.md'

if (-not (Test-Path $versionFile)) {
    if (Test-Path $cargoFile) {
        $cargoVersion = Select-String -Path $cargoFile -Pattern '^version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"' | Select-Object -First 1
        if ($cargoVersion -and $cargoVersion.Matches.Count -gt 0) {
            Set-Content -Path $versionFile -Value $cargoVersion.Matches[0].Groups[1].Value -NoNewline
        } else {
            Set-Content -Path $versionFile -Value '0.1.0' -NoNewline
        }
    } else {
        Set-Content -Path $versionFile -Value '0.1.0' -NoNewline
    }
}

$currentVersion = (Get-Content $versionFile -Raw).Trim()
if ($currentVersion -notmatch '^[0-9]+\.[0-9]+\.[0-9]+$') {
    throw "Invalid version in $versionFile: $currentVersion"
}

$lastTag = (git tag --list 'v*' --sort=-v:refname | Select-Object -First 1)
$range = if ([string]::IsNullOrWhiteSpace($lastTag)) { 'HEAD' } else { "$lastTag..HEAD" }

$logOutput = git log $range --pretty=format:'%s%n%b%n==END=='
$bump = 'none'
foreach ($line in $logOutput) {
    if ($line -eq '==END==') { continue }

    if ($line -match 'BREAKING\s+CHANGE|^(feat|fix)(\([^)]+\))?!:') {
        $bump = 'major'
        break
    }

    if ($line -match '^feat(\([^)]+\))?:') {
        if ($bump -ne 'major') { $bump = 'minor' }
    } elseif ($line -match '^fix(\([^)]+\))?:') {
        if ($bump -eq 'none') { $bump = 'patch' }
    }
}

if ($bump -eq 'none') {
    Write-Host 'No semantic version bump required.'
    exit 0
}

$parts = $currentVersion.Split('.')
$major = [int]$parts[0]
$minor = [int]$parts[1]
$patch = [int]$parts[2]

switch ($bump) {
    'major' { $major++; $minor = 0; $patch = 0 }
    'minor' { $minor++; $patch = 0 }
    'patch' { $patch++ }
}

$newVersion = "$major.$minor.$patch"
Set-Content -Path $versionFile -Value $newVersion -NoNewline

if (Test-Path $cargoFile) {
    $cargoContents = Get-Content $cargoFile -Raw
    $updatedCargo = [regex]::Replace($cargoContents, '^version\s*=\s*"[0-9]+\.[0-9]+\.[0-9]+"', "version = \"$newVersion\"", 1, [System.Text.RegularExpressions.RegexOptions]::Multiline)
    Set-Content -Path $cargoFile -Value $updatedCargo -NoNewline
}

if (-not (Test-Path $changelogFile)) {
    Set-Content -Path $changelogFile -Value "# Changelog`n`nAll notable changes to this project will be documented in this file.`n"
}

$releaseDate = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd')
$logLines = git log $range --pretty=format:'- %s (%h)'
$entry = @("## v$newVersion - $releaseDate") + $logLines + @('', '')
$entryText = $entry -join [Environment]::NewLine

$changelog = Get-Content $changelogFile -Raw
if ($changelog -notmatch "(?m)^## v$newVersion\b") {
    $lines = Get-Content $changelogFile
    $header = $lines | Select-Object -First 3
    $tail = $lines | Select-Object -Skip 3

    $combined = @()
    $combined += $header
    $combined += $entry
    $combined += $tail

    Set-Content -Path $changelogFile -Value ($combined -join [Environment]::NewLine)
}

Write-Host "Bumped $currentVersion -> $newVersion ($bump)"
