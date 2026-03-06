@echo off
setlocal

:: Determine script directory
set "SCRIPT_DIR=%~dp0"
for %%I in ("%SCRIPT_DIR%.") do set "SCRIPT_DIR=%%~fI"

:: Locate the root directory by checking for the gui directory
if exist "%SCRIPT_DIR%\gui" (
    set "ROOT_DIR=%SCRIPT_DIR%"
) else if exist "%SCRIPT_DIR%\..\gui" (
    for %%I in ("%SCRIPT_DIR%\..") do set "ROOT_DIR=%%~fI"
) else (
    echo [ERROR] Could not locate gui directory
    pause
    exit /b 1
)

set "VENV_DIR=%ROOT_DIR%\.venv"
set "PYTHON_VENV=%VENV_DIR%\Scripts\python.exe"
set "GUI_SCRIPT=%ROOT_DIR%\gui\main.py"

:: Check if virtual environment exists
if not exist "%PYTHON_VENV%" (
    echo [ERROR] Python virtual environment not found. Please run install.bat first.
    pause
    exit /b 1
)

:: Run Python GUI script
"%PYTHON_VENV%" "%GUI_SCRIPT%"
