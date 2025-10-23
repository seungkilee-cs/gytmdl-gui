#!/usr/bin/env python3
"""
Entry point script for gytmdl sidecar binary.
This script properly initializes the gytmdl module and handles imports correctly.
"""

import sys
import os
from pathlib import Path

def main():
    """Main entry point for gytmdl sidecar."""
    try:
        # Import and run gytmdl
        import gytmdl.cli
        gytmdl.cli.main()
    except ImportError as e:
        print(f"Error importing gytmdl: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Error running gytmdl: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()