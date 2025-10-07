# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.1] - 2025-10-08

### 🎨 Improved User Experience

#### Changed
- **Default output format**: Changed from JSON to Table for better readability
- **Concise output**: `py-license-auditor check` now shows clean, formatted table by default
- **JSON still available**: Use `--format json` when JSON output is needed

#### Fixed
- **E2E tests**: Updated GitHub Actions workflow for new subcommand structure
- **CLI compatibility**: Fixed test failures with new default format

#### Developer Experience
- More user-friendly default output for terminal usage
- Better first-time user experience with readable table format
- Maintains programmatic access via explicit `--format json`

## [0.5.0] - 2025-10-08

### 🎯 Ruff-like Subcommand Structure

#### Added
- **BREAKING**: New subcommand-based CLI structure inspired by ruff
- `check` subcommand: Run license audit on packages (replaces default behavior)
- `init` subcommand: Initialize configuration with preset policy (replaces `--init`)
- `fix` subcommand: Automatically add violations as exceptions (replaces `--interactive`)
- `config` subcommand: Show or validate configuration
- Global options: `--verbose` and `--quiet` for all subcommands
- Position argument support for path in `check` and `fix` subcommands
- `--dry-run` mode for `fix` subcommand to preview changes
- `--validate` option for `config` subcommand
- `--exit-zero` option for `check` subcommand (CI-friendly)

#### Changed
- **BREAKING**: All CLI usage now requires explicit subcommands
- **BREAKING**: `--init <policy>` → `init <policy>`
- **BREAKING**: `--interactive` → `fix` subcommand with automatic exception handling
- **BREAKING**: `--path <path>` → positional argument `[PATH]`
- Improved error messages and help text
- Better quiet mode support across all subcommands

#### Migration Guide
```bash
# Before (v0.4.x)
py-license-auditor --init green
py-license-auditor --format json --output report.json
py-license-auditor --interactive

# After (v0.5.0)
py-license-auditor init green
py-license-auditor check --format json --output report.json
py-license-auditor fix --dry-run  # Preview changes
py-license-auditor fix            # Apply exceptions
```

#### Developer Experience
- Familiar ruff-like interface for Python developers
- Clear separation of concerns between subcommands
- Better discoverability with `py-license-auditor help <command>`
- Improved CI/CD integration with focused `check` command

## [0.4.1] - 2025-10-02

### 🚦 Traffic Light Policy System

#### Changed
- **BREAKING**: Policy names changed from `personal/corporate/ci` to `green/yellow/red`
- **Intuitive naming**: Traffic light system for clear risk indication
- **Consistent fail behavior**: Green/Yellow fail on violations, Red continues for audit

#### Policy Mapping
- `green` (was `corporate`): Safe for commercial use - MIT, Apache, BSD, ISC only
- `yellow` (was `ci`): Balanced policy - adds LGPL, MPL weak copyleft licenses  
- `red` (was `personal`): Audit mode - all licenses allowed, `fail_on_violations = false`

#### Migration Guide
```bash
# Before (v0.4.0)
py-license-auditor --init corporate

# After (v0.4.1)  
py-license-auditor --init green
```

### 🎯 Policy Clarity
- **Green**: Enterprise/commercial development (strictest)
- **Yellow**: Balanced development (moderate restrictions)
- **Red**: Audit/OSS development (information gathering)

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
