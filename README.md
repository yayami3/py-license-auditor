# py-license-auditor

[![Crates.io](https://img.shields.io/crates/v/py-license-auditor.svg)](https://crates.io/crates/py-license-auditor)
[![PyPI](https://img.shields.io/pypi/v/py-license-auditor.svg)](https://pypi.org/project/py-license-auditor/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

**The fastest license auditor for uv projects** - Built specifically for the modern Python ecosystem.

> 🎯 **uv-First Strategy**: This tool is designed exclusively for [uv](https://github.com/astral-sh/uv) projects. We focus on providing the best possible experience for uv users rather than supporting all package managers.

## ✨ Why uv + py-license-auditor?

- 🚀 **Built for Speed**: Both tools are written in Rust for maximum performance
- 🎯 **uv-Native**: Deep integration with `uv.lock` and uv workflows  
- 🔧 **Zero Config**: Works out of the box with uv projects
- ⚡ **Fast Workflow**: `uv sync && py-license-auditor` - that's it!

## 🚀 Installation

### For uv Users (Recommended)
```bash
# Install as a uv tool
uv tool install py-license-auditor

# Use in any uv project
cd my-uv-project
uv tool run py-license-auditor
```

### Manual Installation
Download the binary for your platform from [GitHub Releases](https://github.com/yayami3/py-license-auditor/releases/latest).

### From Source
```bash
git clone https://github.com/yayami3/py-license-auditor
cd py-license-auditor
cargo install --path .
```

## 📖 Usage

> 📚 **Quick Start**: See [QUICKSTART.md](QUICKSTART.md) for a step-by-step guide

### Quick Start
```bash
# 1. Setup your uv project
uv init my-project
cd my-project
uv add requests pandas

# 2. Configure license policy (one-time setup)
py-license-auditor --init green

# 3. Run license audit
uv sync
py-license-auditor
```

### Configuration Setup

#### Initialize with Built-in Policies
```bash
# For commercial/enterprise projects (safest)
py-license-auditor --init green

# For balanced development (permissive + weak copyleft)
py-license-auditor --init yellow

# For audit/OSS development (information gathering)
py-license-auditor --init red
```

This creates a `[tool.py-license-auditor]` section in your `pyproject.toml` with appropriate settings.

### Basic Usage
```bash
# Auto-detect .venv in current directory
py-license-auditor

# Specify site-packages directory
py-license-auditor --path /path/to/site-packages

# Save to file
py-license-auditor --output licenses.json
```

### Output Formats
```bash
# JSON (default)
py-license-auditor --format json

# Table for terminal viewing
py-license-auditor --format table

# CSV for spreadsheets
py-license-auditor --format csv
```

### Advanced Options
```bash
# Include packages without license info
py-license-auditor --include-unknown

# Combine options
py-license-auditor --format csv --output report.csv --include-unknown

# Interactive mode for handling violations
py-license-auditor --interactive
```

## 📊 Output Example

### JSON Format
```json
{
  "packages": [
    {
      "name": "requests",
      "version": "2.31.0",
      "license": "Apache-2.0",
      "license_classifiers": [
        "License :: OSI Approved :: Apache Software License"
      ],
      "metadata_source": "METADATA"
    }
  ],
  "summary": {
    "total_packages": 50,
    "with_license": 45,
    "without_license": 5,
    "license_types": {
      "osi_approved": {
        "MIT": 20,
        "Apache-2.0": 15,
        "BSD": 8
      },
      "non_osi": {
        "MIT License": 2
      }
    }
  },
  "violations": {
    "total": 2,
    "errors": 1,
    "warnings": 1,
    "details": [
      {
        "package_name": "some-gpl-lib",
        "package_version": "2.1.0",
        "license": "GPL-3.0",
        "violation_level": "Forbidden",
        "matched_rule": "exact: GPL-3.0",
        "message": "License 'GPL-3.0' is forbidden by policy"
      }
    ]
  }
}
```

### CSV Format
```csv
name,version,license,license_classifiers,metadata_source
requests,2.31.0,Apache-2.0,"License :: OSI Approved :: Apache Software License",METADATA
click,8.1.7,BSD-3-Clause,"License :: OSI Approved :: BSD License",METADATA
```

## 🎛️ Policy Configuration

### Built-in Policies

Three ready-to-use policies are included:

```bash
# Green: Safe for commercial use - only permissive licenses
py-license-auditor --init green

# Yellow: Balanced policy - permissive + weak copyleft
py-license-auditor --init yellow

# Red: Audit mode - all licenses allowed for information gathering
py-license-auditor --init red
```

| Policy | Allowed | Forbidden | Review Required | Fails on Violation |
|--------|---------|-----------|-----------------|-------------------|
| **Green** | MIT, Apache-2.0, BSD-*, ISC | GPL-*, AGPL-*, LGPL-*, MPL-2.0 | None | Yes |
| **Yellow** | MIT, Apache-2.0, BSD-*, ISC, LGPL-*, MPL-2.0 | GPL-*, AGPL-* | None | Yes |
| **Red** | MIT, Apache-2.0, BSD-*, ISC, LGPL-*, MPL-2.0 | None | GPL-*, AGPL-* | No |

### Custom Policy Configuration

After running `py-license-auditor --init`, you can customize the generated configuration in `pyproject.toml`:

```toml
[tool.py-license-auditor]
format = "json"
include_unknown = true
fail_on_violations = true

[tool.py-license-auditor.policy]
name = "Custom License Policy"
description = "Tailored policy for our project"

[tool.py-license-auditor.policy.allowed_licenses]
exact = ["MIT", "Apache-2.0", "BSD-3-Clause", "ISC"]
patterns = ["BSD-*"]

[tool.py-license-auditor.policy.forbidden_licenses]
exact = ["GPL-3.0", "AGPL-3.0"]
patterns = ["GPL-*", "AGPL-*"]

[tool.py-license-auditor.policy.review_required]
exact = ["MPL-2.0"]
patterns = ["LGPL-*"]

[[tool.py-license-auditor.policy.exceptions]]
name = "legacy-package"
version = "1.0.0"
reason = "Approved by legal team for legacy compatibility"
```

### Policy Rules

- **allowed_licenses**: Licenses that are automatically approved
- **forbidden_licenses**: Licenses that cause build failures
- **review_required**: Licenses that need manual review (warnings)
- **exceptions**: Package-specific overrides with justification

### Pattern Matching

Use glob patterns for flexible license matching:
- `"GPL-*"` matches `GPL-2.0`, `GPL-3.0`, etc.
- `"BSD-*"` matches `BSD-2-Clause`, `BSD-3-Clause`, etc.

## 🎯 Use Cases

### License Compliance
Generate comprehensive reports for legal review and compliance auditing.

```bash
# Generate compliance report
py-license-auditor --format json --output compliance-report.json
```

### CI/CD Integration
Automate license checking in your deployment pipeline.

```yaml
# GitHub Actions example
- name: Setup License Policy
  run: py-license-auditor --init green
  
- name: License Check  
  run: py-license-auditor
    
- name: Generate License Report
  run: py-license-auditor --format json --output license-report.json
```

### Dependency Auditing
Understand your project's license obligations and risks.

```bash
# Focus on potential issues
py-license-auditor --format json
```

## 🔍 License Categories

The tool categorizes licenses into two groups:

- **OSI Approved**: Licenses approved by the Open Source Initiative (legally vetted)
- **Non-OSI**: Custom licenses, proprietary licenses, or unrecognized formats

This helps you quickly identify which licenses need manual legal review.

## 🛠️ Development

### Building from Source
```bash
git clone https://github.com/yayami3/py-license-auditor
cd py-license-auditor
cargo build --release
```

### Running Tests
```bash
cargo test
```

### Contributing
Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

- Built with [Clap](https://github.com/clap-rs/clap) for CLI parsing
- Uses [Serde](https://github.com/serde-rs/serde) for serialization
- Inspired by the need for better Python license compliance tools
