# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-10-02

### 🎉 Major Architecture Redesign

#### Added
- **Embedded Policy Configuration**: Policy settings now live in `pyproject.toml`
- **Simplified CLI**: Streamlined to essential options only
- **Policy Initialization**: `--init` command for easy setup with built-in presets
- **Interactive Mode**: `--interactive` for handling violations interactively
- **Comprehensive E2E Tests**: Full end-to-end test suite with proper Rust structure
- **GitHub Actions Workflow**: Automated E2E testing in CI/CD

#### Changed
- **BREAKING**: Removed `--policy`, `--check-violations`, `--fail-on-violations` CLI options
- **BREAKING**: Policy configuration moved from separate files to `pyproject.toml`
- **Workflow**: New 2-step process: `--init` once, then run repeatedly
- **Configuration**: All settings now centralized in `pyproject.toml`

#### Removed
- **BREAKING**: Built-in policy CLI arguments
- **BREAKING**: Separate policy file support (`--policy-file`)
- **BREAKING**: Command-line policy overrides

#### Fixed
- Unused variable warnings in compilation
- Unnecessary Python dependencies in `pyproject.toml`
- Module structure for E2E tests following Rust conventions

### 🚀 Migration Guide

#### Before (v0.3.x)
```bash
py-license-auditor --policy corporate --check-violations
```

#### After (v0.4.0)
```bash
# One-time setup
py-license-auditor --init corporate

# Regular usage
py-license-auditor
```

### 📊 Technical Improvements
- **Code Reduction**: -217 +191 lines (net simplification)
- **Test Coverage**: 20 unit tests + 4 E2E tests
- **Zero Warnings**: Clean compilation
- **Better Documentation**: Updated README and QUICKSTART guides

## [0.3.3] - 2025-09-30

### Added
- Initial uv.lock integration
- Basic policy system
- Multiple output formats

### Fixed
- License detection improvements
- Error handling enhancements
