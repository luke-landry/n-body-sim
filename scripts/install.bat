@echo off
setlocal

:: Determine script directory
set "SCRIPT_DIR=%~dp0"
for %%I in ("%SCRIPT_DIR%.") do set "SCRIPT_DIR=%%~fI"

:: Locate the root directory by checking for requirements.txt
if exist "%SCRIPT_DIR%\requirements.txt" (
    set "ROOT_DIR=%SCRIPT_DIR%"
) else if exist "%SCRIPT_DIR%\..\requirements.txt" (
    for %%I in ("%SCRIPT_DIR%\..") do set "ROOT_DIR=%%~fI"
) else (
    echo [ERROR] Could not locate requirements.txt
    pause
    exit /b 1
)

set "VENV_DIR=%ROOT_DIR%\.venv"
set "PYTHON_VENV=%VENV_DIR%\Scripts\python.exe"
set "REQ_FILE=%ROOT_DIR%\requirements.txt"

:: Check Python is installed
where python >nul 2>&1
if errorlevel 1 (
    where py >nul 2>&1
    if errorlevel 1 (
        echo [ERROR] Python is not installed or not added to your PATH.
        pause
        exit /b 1
    )
    set "PYTHON=py"
) else (
    set "PYTHON=python"
)

:: Setup venv
if not exist "%PYTHON_VENV%" (
    echo [INFO] Creating Python virtual environment...
    "%PYTHON%" -m venv "%VENV_DIR%"
    if errorlevel 1 (
        echo [ERROR] Failed to create Python virtual environment.
        pause
        exit /b 1
    )
)

:: Check requirements
if not exist "%REQ_FILE%" (
    echo [ERROR] %REQ_FILE% not found.
    pause
    exit /b 1
)

:: Ensure pip is available
"%PYTHON_VENV%" -m pip --version >nul 2>&1
if errorlevel 1 (
    echo [INFO] Bootstrapping pip...
    "%PYTHON_VENV%" -m ensurepip --upgrade
    if errorlevel 1 (
        echo [ERROR] Failed to install pip.
        pause
        exit /b 1
    )
)

:: Upgrade pip
echo [INFO] Upgrading pip in virtual environment...
"%PYTHON_VENV%" -m pip install --upgrade pip
if errorlevel 1 (
    echo [ERROR] Failed to upgrade pip.
    pause
    exit /b 1
)

:: Install packages
echo [INFO] Installing packages...
"%PYTHON_VENV%" -m pip install -r "%REQ_FILE%"
if errorlevel 1 (
    echo [ERROR] Failed to install packages.
    pause
    exit /b 1
)

echo [SUCCESS] Installation complete. Use run.bat to start the application.
pause
exit /b 0
