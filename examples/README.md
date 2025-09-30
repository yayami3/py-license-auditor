# Configuration Examples

This directory contains example configuration files for different use cases.

## Quick Start

Copy one of these files to your project's `pyproject.toml`:

```bash
# Personal/Open Source projects
py-license-auditor --init personal

# Corporate/Proprietary projects  
py-license-auditor --init corporate

# CI/CD pipelines
py-license-auditor --init ci
```

## Example Files

### [`personal.toml`](personal.toml)
- **Use case**: Individual developers, open source projects
- **Policy**: Permissive, allows most OSI licenses
- **Output**: Table format for human reading
- **Violations**: Warnings only, no build failures

### [`corporate.toml`](corporate.toml)
- **Use case**: Companies, proprietary software
- **Policy**: Conservative, strict license restrictions
- **Output**: JSON format for processing
- **Violations**: Build failures on forbidden licenses

### [`ci.toml`](ci.toml)
- **Use case**: Automated CI/CD pipelines
- **Policy**: Minimal, fast approval process
- **Output**: JSON format for automation
- **Violations**: Build failures, optimized for speed

### [`comprehensive.toml`](comprehensive.toml)
- **Use case**: Reference for all available options
- **Policy**: Demonstrates every configuration feature
- **Output**: Shows advanced patterns and exceptions

## Manual Configuration

To manually configure, add a `[tool.py-license-auditor]` section to your `pyproject.toml`:

```toml
[tool.py-license-auditor]
format = "table"
include_unknown = false

[tool.py-license-auditor.policy]
name = "My Custom Policy"
allowed_licenses = ["MIT", "Apache-2.0"]
forbidden_licenses = ["GPL-*"]
```

## Configuration Options

| Option | Type | Description | Default |
|--------|------|-------------|---------|
| `format` | string | Output format: `table`, `json`, `toml`, `csv` | `table` |
| `include_unknown` | boolean | Include packages without license info | `false` |
| `fail_on_violations` | boolean | Exit with error on violations | `false` |
| `check_violations` | boolean | Enable violation checking | `false` |
| `output` | string | Output file path (optional) | stdout |

## Policy Configuration

| Policy Option | Type | Description |
|---------------|------|-------------|
| `name` | string | Policy name for reports |
| `description` | string | Policy description (optional) |
| `allowed_licenses` | array | Exact license matches |
| `allowed_patterns` | array | Glob patterns for allowed licenses |
| `forbidden_licenses` | array | Exact forbidden license matches |
| `forbidden_patterns` | array | Glob patterns for forbidden licenses |
| `review_required` | array | Licenses requiring manual review |
| `review_patterns` | array | Glob patterns for review licenses |

## Exception Handling

```toml
[[tool.py-license-auditor.exceptions]]
name = "package-name"
version = "1.0.0"  # Optional, defaults to any version
reason = "Justification for exception"
```

## Pattern Matching

Use glob patterns for flexible license matching:

- `"GPL-*"` matches `GPL-2.0`, `GPL-3.0`, etc.
- `"BSD-*"` matches `BSD-2-Clause`, `BSD-3-Clause`, etc.
- `"*GPL*"` matches `LGPL-2.1`, `AGPL-3.0`, etc.

## Integration Examples

### GitHub Actions
```yaml
- name: License Check
  run: py-license-auditor --check-violations --fail-on-violations
```

### Pre-commit Hook
```yaml
- repo: local
  hooks:
    - id: license-check
      name: License Compliance
      entry: py-license-auditor --check-violations
      language: system
```

### Make Target
```makefile
license-check:
	py-license-auditor --format json --output licenses.json
	py-license-auditor --check-violations --fail-on-violations
```
