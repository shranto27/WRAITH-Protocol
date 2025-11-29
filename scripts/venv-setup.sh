#!/usr/bin/env bash
# Quick fix script for Python venv issues in WRAITH Protocol project

set -euo pipefail

PROJECT_DIR="/home/parobek/Code/WRAITH-Protocol"
VENV_DIR="$PROJECT_DIR/.venv"

echo "=== Python venv Diagnostic and Fix Script ==="
echo "Project: WRAITH Protocol"
echo "Date: $(date)"
echo ""

# Check Python availability
echo "1. Checking Python installation..."
if ! command -v python3 &> /dev/null; then
    echo "ERROR: python3 not found"
    exit 1
fi
echo "   ✓ Python: $(python3 --version)"

# Check venv module
echo "2. Checking venv module..."
if ! python3 -m venv --help &> /dev/null; then
    echo "ERROR: venv module not available"
    echo "SOLUTION: Install python-venv package"
    exit 1
fi
echo "   ✓ venv module available"

# Check if venv exists
echo "3. Checking virtual environment..."
if [ -d "$VENV_DIR" ]; then
    echo "   ✓ venv exists at $VENV_DIR"
    
    # Test activation
    echo "4. Testing venv activation..."
    if source "$VENV_DIR/bin/activate" && command -v pip &> /dev/null; then
        echo "   ✓ venv activation successful"
        echo "   ✓ pip available: $(pip --version | head -1)"
        
        # Check yamllint
        echo "5. Checking yamllint..."
        if command -v yamllint &> /dev/null; then
            echo "   ✓ yamllint installed: $(yamllint --version)"
        else
            echo "   ! yamllint not found, installing..."
            pip install yamllint
            echo "   ✓ yamllint installed"
        fi
        
        deactivate
    else
        echo "   ERROR: venv activation failed"
        echo "   Recreating venv..."
        rm -rf "$VENV_DIR"
        python3 -m venv "$VENV_DIR"
        source "$VENV_DIR/bin/activate"
        pip install --upgrade pip
        pip install yamllint
        echo "   ✓ venv recreated successfully"
        deactivate
    fi
else
    echo "   ! venv does not exist, creating..."
    python3 -m venv "$VENV_DIR"
    source "$VENV_DIR/bin/activate"
    pip install --upgrade pip
    pip install yamllint
    echo "   ✓ venv created successfully"
    deactivate
fi

echo ""
echo "=== Diagnostic Complete ==="
echo "Status: ✓ All checks passed"
echo ""
echo "Usage:"
echo "  source $VENV_DIR/bin/activate && yamllint .github/"
echo ""
