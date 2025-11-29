# Python Tooling Guide

This document covers Python tooling used in WRAITH Protocol for auxiliary tasks.

## Overview

WRAITH Protocol is a Rust project. Python is used only for auxiliary tooling:
- YAML linting (yamllint)
- Documentation generation
- Scripting utilities

## Virtual Environment

### Location
```
/home/parobek/Code/WRAITH-Protocol/.venv
```

### Setup (if needed)
```bash
cd /home/parobek/Code/WRAITH-Protocol
python3 -m venv .venv
source .venv/bin/activate && pip install --upgrade pip
source .venv/bin/activate && pip install yamllint
```

### Current Packages
- yamllint 1.37.1
- PyYAML 6.0.3
- pathspec 0.12.1

## Usage Patterns

### CRITICAL: Command Chaining

The Claude Code Bash tool runs each command in a separate shell. Always chain commands with `&&`:

```bash
# ✓ CORRECT
source .venv/bin/activate && yamllint .github/
source .venv/bin/activate && pip install <package>

# ❌ WRONG (venv won't be active in second command)
source .venv/bin/activate
pip install <package>
```

### Common Operations

**YAML Linting:**
```bash
source .venv/bin/activate && yamllint .github/
source .venv/bin/activate && yamllint --strict .github/workflows/
```

**Package Management:**
```bash
source .venv/bin/activate && pip list
source .venv/bin/activate && pip install <package>
source .venv/bin/activate && pip freeze > requirements.txt
```

**Health Check:**
```bash
source .venv/bin/activate && python --version && pip --version && yamllint --version
```

## Alternatives

If Python venv is problematic, alternatives exist:

1. **System Package Manager:**
   ```bash
   # CachyOS/Arch
   sudo pacman -S yamllint
   ```

2. **pipx (isolated installation):**
   ```bash
   pipx install yamllint
   ```

3. **Skip Python entirely:**
   - Online YAML validators
   - GitHub Actions pre-configured linters
   - Rust-based validation (serde_yaml in tests)

## Troubleshooting

### "pip: command not found" after activation
Ensure commands are chained:
```bash
source .venv/bin/activate && pip --version
```

### Permission errors
The venv is user-owned, no sudo needed:
```bash
source .venv/bin/activate && pip install <package>
```

### Recreate venv if corrupted
```bash
rm -rf .venv
python3 -m venv .venv
source .venv/bin/activate && pip install --upgrade pip yamllint
```

## Automated Setup Script

A setup script is available to diagnose and fix common venv issues:

```bash
bash scripts/venv-setup.sh
```

This script will:
- Check Python installation
- Verify venv module availability
- Create or repair the virtual environment
- Install required packages (yamllint)
- Validate the installation

## CI/CD Integration

For automated YAML linting in GitHub Actions:

```yaml
- name: Set up Python
  uses: actions/setup-python@v4
  with:
    python-version: '3.13'

- name: Install yamllint
  run: |
    python -m venv .venv
    source .venv/bin/activate
    pip install yamllint

- name: Lint YAML
  run: |
    source .venv/bin/activate
    yamllint .github/
```

## Project-Specific Details

### WRAITH Protocol venv State
- **Location:** `/home/parobek/Code/WRAITH-Protocol/.venv`
- **Python Version:** 3.13.7
- **yamllint Version:** 1.37.1
- **Target Files:** YAML files in `.github/` directory (5 workflow files)

### Quick Health Check
```bash
cd /home/parobek/Code/WRAITH-Protocol
source .venv/bin/activate && \
  python --version && \
  pip --version && \
  yamllint --version && \
  echo "✓ venv healthy"
```

## Key Takeaways

1. **Always use `&&` to chain commands** when using Bash tool with venv
2. **venv activation doesn't persist** across separate bash invocations
3. **Check working directory** before venv operations
4. **Use absolute paths** when in doubt
5. **Automated script available** at `scripts/venv-setup.sh` for diagnostics

---

**Last Updated:** 2025-11-29
**Status:** ✓ Fully Operational
