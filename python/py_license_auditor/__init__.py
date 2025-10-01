"""
py-license-auditor: Fast license auditor for uv projects

This package provides a Python wrapper around the Rust-based py-license-auditor binary,
enabling installation and usage through uv tool commands.
"""

import os
import sys
import subprocess
import platform
from pathlib import Path

__version__ = "0.4.1"

def get_binary_path():
    """Get the path to the py-license-auditor binary for the current platform."""
    package_dir = Path(__file__).parent
    bin_dir = package_dir / "bin"
    
    system = platform.system().lower()
    machine = platform.machine().lower()
    
    # Map platform to binary name
    if system == "linux":
        if machine in ["x86_64", "amd64"]:
            binary_name = "py-license-auditor-linux-x86_64"
        elif machine in ["aarch64", "arm64"]:
            binary_name = "py-license-auditor-linux-aarch64"
        else:
            raise RuntimeError(f"Unsupported Linux architecture: {machine}")
    elif system == "darwin":
        if machine in ["x86_64", "amd64"]:
            binary_name = "py-license-auditor-macos-x86_64"
        elif machine in ["aarch64", "arm64"]:
            binary_name = "py-license-auditor-macos-aarch64"
        else:
            raise RuntimeError(f"Unsupported macOS architecture: {machine}")
    elif system == "windows":
        if machine in ["x86_64", "amd64"]:
            binary_name = "py-license-auditor-windows-x86_64.exe"
        else:
            raise RuntimeError(f"Unsupported Windows architecture: {machine}")
    else:
        raise RuntimeError(f"Unsupported operating system: {system}")
    
    binary_path = bin_dir / binary_name
    
    if not binary_path.exists():
        raise RuntimeError(f"Binary not found: {binary_path}")
    
    return binary_path

def main():
    """Main entry point for the py-license-auditor command."""
    try:
        binary_path = get_binary_path()
        
        # Make sure the binary is executable
        os.chmod(binary_path, 0o755)
        
        # Execute the binary with all arguments
        result = subprocess.run([str(binary_path)] + sys.argv[1:])
        sys.exit(result.returncode)
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
