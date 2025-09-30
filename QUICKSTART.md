# Quick Start Guide

## 1. Basic License Extraction

```bash
# Extract licenses from current virtual environment
py-license-auditor

# Extract from specific path
py-license-auditor --path /path/to/site-packages

# Save to file
py-license-auditor --output licenses.json
```

## 2. License Policy Setup

### Setup Built-in Policies (Recommended)

```bash
# Corporate policy (strict for proprietary software)
py-license-auditor --init corporate

# Permissive policy (balanced for open source)
py-license-auditor --init personal

# Strict policy (very restrictive for CI/CD)
py-license-auditor --init ci
```

### Run License Checking

```bash
# Run with configured policy
py-license-auditor

# Interactive mode for handling violations
py-license-auditor --interactive
```

## 3. Custom Policy Configuration

After running `--init`, customize your policy in `pyproject.toml`:

```toml
[tool.py-license-auditor]
format = "json"
include_unknown = true
check_violations = true
fail_on_violations = true

[tool.py-license-auditor.policy]
name = "My Project Policy"
description = "Custom policy for our project"

[tool.py-license-auditor.policy.allowed_licenses]
exact = ["MIT", "Apache-2.0", "BSD-3-Clause"]
patterns = ["BSD-*"]

[tool.py-license-auditor.policy.forbidden_licenses]
exact = ["GPL-3.0", "AGPL-3.0"]
patterns = ["GPL-*"]

[tool.py-license-auditor.policy.review_required]
exact = ["MPL-2.0"]
patterns = ["LGPL-*"]
```

## 4. CI/CD Integration

### GitHub Actions

```yaml
name: License Check
on: [push, pull_request]

jobs:
  license-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install py-license-auditor
        run: cargo install py-license-auditor
      - name: Setup License Policy
        run: py-license-auditor --init corporate
      - name: Check licenses
        run: py-license-auditor --output license-report.json
```

## 5. Output Formats

```bash
# JSON (default)
py-license-auditor --format json

# Table for terminal viewing
py-license-auditor --format table

# CSV for spreadsheets
py-license-auditor --format csv
```

## 6. Policy Comparison

| Use Case | Recommended Policy | Setup Command |
|----------|-------------------|---------------|
| Corporate/Proprietary | `corporate` | `py-license-auditor --init corporate` |
| Open Source Project | `personal` | `py-license-auditor --init personal` |
| Maximum Security | `ci` | `py-license-auditor --init ci` |
| Custom Requirements | Custom config | Edit `pyproject.toml` after init |
