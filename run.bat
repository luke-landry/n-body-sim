@echo off
setlocal

:: Set variables
set "VENV_DIR=.venv"
set "PYTHON_VENV=%VENV_DIR%\Scripts\python.exe"
set "SCRIPT=gui\main.py"

:: Check if virtual environment exists
if not exist "%PYTHON_VENV%" (
    echo [ERROR] Python virtual environment not found. Please run install.bat first.
    pause
    exit /b 1
)

:: Run Python script
"%PYTHON_VENV%" "%SCRIPT%"
