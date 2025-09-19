# uv-license-extractor

A command-line tool to extract license information from Python packages installed in your environment.

## Features

- Extract license information from `.dist-info` and `.egg-info` directories
- Support for multiple output formats (JSON, TOML, CSV)
- Automatic detection of virtual environments
- Summary statistics of license usage
- Filter packages with/without license information

## Installation

```bash
cargo install --path .
```

## Usage

### Basic usage (auto-detect .venv)
```bash
uv-license-extractor
```

### Specify site-packages directory
```bash
uv-license-extractor --path /path/to/site-packages
```

### Different output formats
```bash
# JSON (default)
uv-license-extractor --format json

# TOML
uv-license-extractor --format toml

# CSV
uv-license-extractor --format csv
```

### Save to file
```bash
uv-license-extractor --output licenses.json
```

### Include packages without license info
```bash
uv-license-extractor --include-unknown
```

## Output Example

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
      "MIT": 20,
      "Apache-2.0": 15,
      "BSD": 10
    }
  }
}
```

### CSV Format
```csv
name,version,license,license_classifiers,metadata_source
requests,2.31.0,Apache-2.0,"License :: OSI Approved :: Apache Software License",METADATA
click,8.1.7,BSD-3-Clause,"License :: OSI Approved :: BSD License",METADATA
```

## Use Cases

- **License Compliance**: Generate reports for legal review
- **Dependency Auditing**: Understand license obligations
- **CI/CD Integration**: Automated license checking
- **Documentation**: Include license information in project docs

## License

This project is licensed under MIT OR Apache-2.0.
