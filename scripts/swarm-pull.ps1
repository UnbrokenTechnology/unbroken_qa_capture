# swarm-pull.ps1 — Pull latest agent work from the swarm bare repo
# Usage: powershell -ExecutionPolicy Bypass -File scripts/swarm-pull.ps1 [-Run]
#   -Run    Also launch `npm run tauri:dev` after pulling

param(
    [switch]$Run
)

$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $root

Write-Host "`n=== Swarm Pull ===" -ForegroundColor Cyan

# 1. Kill running app so the exe isn't locked
$procs = Get-Process -Name "unbroken-qa-capture" -ErrorAction SilentlyContinue
if ($procs) {
    Write-Host "Stopping running app..." -ForegroundColor Yellow
    $procs | Stop-Process -Force
    Start-Sleep -Seconds 2
}

# 2. Show what's new before pulling
Write-Host "`nFetching swarm/main..." -ForegroundColor Cyan
$env:GIT_REDIRECT_STDERR = '2>&1'
git fetch swarm main 2>$null

$local  = (git rev-parse HEAD).Trim()
$remote = (git rev-parse swarm/main).Trim()

if ($local -eq $remote) {
    Write-Host "Already up to date." -ForegroundColor Green
    if ($Run) {
        Write-Host "`nStarting dev server..." -ForegroundColor Cyan
        & npm run tauri:dev
    }
    exit 0
}

$commits = git log --oneline "$local..$remote"
$count = ($commits | Measure-Object).Count
Write-Host "$count new commit(s):" -ForegroundColor Green
$commits | ForEach-Object { Write-Host "  $_" }

# 3. Fast-forward merge
Write-Host "`nPulling (fast-forward only)..." -ForegroundColor Cyan
git pull swarm main --ff-only 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Fast-forward failed. Local main has diverged." -ForegroundColor Red
    Write-Host "Resolve manually: git merge swarm/main or git rebase swarm/main" -ForegroundColor Yellow
    exit 1
}

# 4. Sync to GitHub
Write-Host "`nPushing to origin..." -ForegroundColor Cyan
git push origin main 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "WARNING: Push to origin failed (non-fatal)." -ForegroundColor Yellow
}

Write-Host "`nDone. Pulled $count commit(s)." -ForegroundColor Green

# 5. Optionally run
if ($Run) {
    Write-Host "`nStarting dev server..." -ForegroundColor Cyan
    & npm run tauri:dev
}
