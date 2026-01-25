#!/bin/bash
set -e

VENV_DIR=".venv"
PYTHON_VENV="$VENV_DIR/bin/python"
REQ_FILE="requirements.txt"

# Check Python is installed
if command -v python3 >/dev/null 2>&1; then
    PYTHON="python3"
elif command -v python >/dev/null 2>&1; then
    PYTHON="python"
else
    echo "[ERROR] Python is not installed or not added to your PATH."
    exit 1
fi

# Setup virtual environment
if [ ! -f "$PYTHON_VENV" ]; then
    echo "[INFO] Creating virtual environment..."
    "$PYTHON" -m venv "$VENV_DIR" || { echo "[ERROR] Failed to create virtual environment."; exit 1; }
fi

# Check requirements
if [ ! -f "$REQ_FILE" ]; then
    echo "[ERROR] $REQ_FILE not found."
    exit 1
fi

# Ensure pip is available
if ! "$PYTHON_VENV" -m pip --version >/dev/null 2>&1; then
    echo "[INFO] Bootstrapping pip..."
    "$PYTHON_VENV" -m ensurepip --upgrade || { echo "[ERROR] Failed to install pip."; exit 1; }
fi

# Upgrade pip
echo "[INFO] Upgrading pip in virtual environment..."
"$PYTHON_VENV" -m pip install --upgrade pip || { echo "[ERROR] Failed to upgrade pip."; exit 1; }

# Install packages
echo "[INFO] Installing packages..."
"$PYTHON_VENV" -m pip install -r "$REQ_FILE" || { echo "[ERROR] Failed to install packages."; exit 1; }

echo "[SUCCESS] Installation complete."