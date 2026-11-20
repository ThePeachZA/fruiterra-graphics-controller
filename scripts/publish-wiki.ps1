# Publishes docs/wiki to the GitHub wiki.
# Also syncs docs/wiki/images/ into the wiki when that folder exists.
#
# The GitHub wiki repository only exists after the wiki is enabled and its first
# page has been created in the GitHub web UI. Private repositories need a plan
# that supports private wikis.
#
# Usage:
#   pwsh ./scripts/publish-wiki.ps1
#   pwsh ./scripts/publish-wiki.ps1 -Repo owner/name

param(
    [string]$Repo = "ThePeachZA/fruiterra-graphics-controller",
    [string]$Source = (Join-Path $PSScriptRoot "..\docs\wiki"),
    [string]$WorkDir = (Join-Path $env:TEMP "fgc-wiki"),
    [string]$Message = "docs(wiki): publish end-user guide"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path (Join-Path $Source "Home.md"))) {
    throw "Wiki source not found at $Source"
}

$url = "https://github.com/$Repo.wiki.git"

if (Test-Path (Join-Path $WorkDir ".git")) {
    Write-Host "Updating existing wiki clone at $WorkDir"
    git -C $WorkDir pull --ff-only
} else {
    Write-Host "Cloning $url"
    if (Test-Path $WorkDir) {
        Remove-Item -Recurse -Force $WorkDir
    }
    git clone $url $WorkDir
}

if ($LASTEXITCODE -ne 0) {
    throw @"
Could not clone the wiki repository.

Check that the wiki is enabled:
  gh api repos/$Repo --jq .has_wiki

Private repositories on the GitHub Free plan cannot enable wikis. Make the
repository public, upgrade the plan, or keep using docs/wiki in the repository.
"@
}

Get-ChildItem -Path $Source -Filter *.md -File | ForEach-Object {
    Copy-Item $_.FullName (Join-Path $WorkDir $_.Name) -Force
    Write-Host "Copied $($_.Name)"
}

$imageSource = Join-Path $Source "images"
$imageDest = Join-Path $WorkDir "images"

if (Test-Path $imageSource) {
    if (Test-Path $imageDest) {
        Remove-Item -Recurse -Force $imageDest
    }
    Copy-Item -Path $imageSource -Destination $WorkDir -Recurse -Force
    Write-Host "Synced images from $imageSource"
} elseif (Test-Path $imageDest) {
    Remove-Item -Recurse -Force $imageDest
    Write-Host "Removed stale images from the wiki"
}

git -C $WorkDir add -A

git -C $WorkDir diff --cached --quiet
if ($LASTEXITCODE -eq 0) {
    Write-Host "No wiki changes to publish."
    exit 0
}

git -C $WorkDir commit -m $Message
if ($LASTEXITCODE -ne 0) {
    throw "Could not commit the wiki changes."
}

git -C $WorkDir push
if ($LASTEXITCODE -ne 0) {
    throw "Could not push the wiki changes."
}

Write-Host "Published https://github.com/$Repo/wiki"
