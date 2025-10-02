# PowerShell script for qBittorrent Rule Editor
# This script has better Unicode support than batch files

Write-Host "Starting qBittorrent Rule Editor..." -ForegroundColor Green

# Check if Python is available
try {
    $pythonVersion = python --version 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error: Python not found. Please ensure Python is installed and added to PATH" -ForegroundColor Red
        Read-Host "Press Enter to exit"
        exit 1
    }
    Write-Host "Found: $pythonVersion" -ForegroundColor Yellow
}
catch {
    Write-Host "Error: Python not found. Please ensure Python is installed and added to PATH" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

# Run the editor
Write-Host "Launching editor..." -ForegroundColor Green
python qb_rule_editor.py

Write-Host "Editor closed." -ForegroundColor Yellow
Read-Host "Press Enter to exit"