[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$Manifest
)

$ErrorActionPreference = 'Stop'

$manifestPath = (Resolve-Path -LiteralPath $Manifest).Path
$baseDir = Split-Path -Parent $manifestPath

$failed = 0
$checked = 0

foreach ($line in Get-Content -LiteralPath $manifestPath) {
    # Skip blank lines
    if ([string]::IsNullOrWhiteSpace($line)) {
        continue
    }

    # sha256sum format:
    # <64-character SHA256>  <relative path>
    # Also accepts the binary-mode form with a leading '*'.
    if ($line -notmatch '^([0-9a-fA-F]{64})\s+[* ](.+)$') {
        Write-Warning "Invalid manifest line: $line"
        $failed++
        continue
    }

    $expectedHash = $Matches[1].ToUpperInvariant()
    $relativePath = $Matches[2]

    $filePath = Join-Path -Path $baseDir -ChildPath $relativePath

    if (-not (Test-Path -LiteralPath $filePath -PathType Leaf)) {
        Write-Host "FAILED  $relativePath (file not found)"
        $failed++
        continue
    }

    $actualHash = (Get-FileHash -LiteralPath $filePath -Algorithm SHA256).Hash

    $checked++

    if ($actualHash -eq $expectedHash) {
        Write-Host "OK      $relativePath"
    }
    else {
        Write-Host "FAILED  $relativePath"
        Write-Host "        Expected: $expectedHash"
        Write-Host "        Actual:   $actualHash"
        $failed++
    }
}

Write-Host ""
Write-Host "Checked: $checked"
Write-Host "Failed:  $failed"

if ($failed -gt 0) {
    exit 1
}

exit 0

