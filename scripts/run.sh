#!/bin/bash
set -e

# Determine the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Locate the root directory by checking for gui directory
if [ -d "$SCRIPT_DIR/gui" ]; then
    ROOT_DIR="$SCRIPT_DIR"
elif [ -d "$SCRIPT_DIR/../gui" ]; then
    ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
else
    echo "[ERROR] Could not locate gui directory"
    exit 1
fi

VENV_DIR="$ROOT_DIR/.venv"
PYTHON_VENV="$VENV_DIR/bin/python"
GUI_SCRIPT="$ROOT_DIR/gui/main.py"

# Check if virtual environment exists
if [ ! -f "$PYTHON_VENV" ]; then
    echo "[ERROR] Python virtual environment not found. Please run install.sh first."
    exit 1
fi

# Run Python GUI script
"$PYTHON_VENV" "$GUI_SCRIPT"
