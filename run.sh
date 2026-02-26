#!/bin/bash
set -e

VENV_DIR=".venv"
PYTHON_VENV="$VENV_DIR/bin/python"
SCRIPT="gui/main.py"

# Check if virtual environment exists
if [ ! -f "$PYTHON_VENV" ]; then
    echo "[ERROR] Virtual environment not found. Please run install.sh first."
    exit 1
fi

# Run Python script
"$PYTHON_VENV" "$SCRIPT"
