@echo off
chcp 65001 >nul
echo Starting qBittorrent Rule Editor...

REM Check if Python is available
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: Python not found. Please ensure Python is installed and added to PATH
    pause
    exit /b 1
)

REM Run the editor
python qb_rule_editor.py

pause