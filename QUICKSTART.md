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

## 2. License Violation Detection

### Option A: Use Built-in Policies (Recommended)

```bash
# Corporate policy (strict for proprietary software)
py-license-auditor --policy corporate --check-violations

# Permissive policy (balanced for open source)
py-license-auditor --policy permissive --check-violations

# Strict policy (very restrictive)
py-license-auditor --policy strict --check-violations

# Fail on violations (for CI/CD)
py-license-auditor --policy corporate --check-violations --fail-on-violations
```

### Option B: Custom Policy File

Create `policy.toml`:
```toml
name = "My Project Policy"

[allowed_licenses]
exact = ["MIT", "Apache-2.0", "BSD-3-Clause"]

[forbidden_licenses]
exact = ["GPL-3.0", "AGPL-3.0"]
patterns = ["GPL-*"]
```

```bash
# Use custom policy
py-license-auditor --policy-file policy.toml --check-violations
```

## 3. CI/CD Integration

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
      - name: Check licenses
        run: |
          py-license-auditor \
            --policy corporate \
            --check-violations \
            --fail-on-violations \
            --output license-report.json
```

## 4. Output Formats

```bash
# JSON (default)
py-license-auditor --format json

# TOML
py-license-auditor --format toml

# CSV for spreadsheets
py-license-auditor --format csv
```

## 5. Policy Comparison

| Use Case | Recommended Policy | Command |
|----------|-------------------|---------|
| Corporate/Proprietary | `corporate` | `--policy corporate` |
| Open Source Project | `permissive` | `--policy permissive` |
| Maximum Security | `strict` | `--policy strict` |
| Custom Requirements | Custom file | `--policy-file my-policy.toml` |
