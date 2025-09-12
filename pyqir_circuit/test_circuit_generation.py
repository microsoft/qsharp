#!/usr/bin/env python3
"""
Test script that runs generate_diagram.py against all .ll files in a directory
and reports success/failure for each file.
"""

import argparse
import os
import subprocess
import sys
from pathlib import Path


def test_ll_file(ll_file_path: Path, generate_diagram_script: Path) -> tuple[bool, str]:
    """
    Test a single .ll file with generate_diagram.py
    Returns (success, error_message) tuple.
    If successful, writes JSON output to a .qsc file next to the input file.
    """
    try:
        result = subprocess.run(
            [sys.executable, str(generate_diagram_script), str(ll_file_path)],
            capture_output=True,
            text=True,
            timeout=30,  # 30 second timeout
        )
        if result.returncode == 0:
            # Write JSON output to .qsc file
            output_file = ll_file_path.with_suffix(".qsc")
            output_file.write_text(result.stdout, encoding="utf-8")
            return True, ""
        else:
            # Get the last line of stderr or stdout, which usually contains the main error
            error_lines = (result.stderr.strip() or result.stdout.strip()).split("\n")
            error_msg = (
                error_lines[-1]
                if error_lines and error_lines[-1]
                else f"Exit code {result.returncode}"
            )
            return False, error_msg
    except subprocess.TimeoutExpired:
        return False, "Timeout (30s exceeded)"
    except Exception as e:
        return False, f"Exception: {e}"


def main():
    parser = argparse.ArgumentParser(
        description="Test generate_diagram.py against all .ll files in a directory"
    )
    parser.add_argument("test_dir", help="Directory containing .ll files to test")
    args = parser.parse_args()

    # Path to the test directory
    test_dir = Path(args.test_dir)

    # Path to generate_diagram.py (assume it's in the same directory as this script)
    script_dir = Path(__file__).parent
    generate_diagram_script = script_dir / "generate_diagram.py"

    if not generate_diagram_script.exists():
        print(f"Error: generate_diagram.py not found at {generate_diagram_script}")
        sys.exit(1)

    if not test_dir.exists():
        print(f"Error: Test directory not found at {test_dir}")
        sys.exit(1)

    # Find all .ll files in the directory
    ll_files = list(test_dir.glob("*.ll"))

    if not ll_files:
        print(f"No .ll files found in {test_dir}")
        sys.exit(1)

    print(f"Testing {len(ll_files)} .ll files in {test_dir}")
    print("=" * 60)

    success_count = 0
    failure_count = 0

    for ll_file in sorted(ll_files):
        print(f"Testing {ll_file.name}...", end=" ")

        success, error_msg = test_ll_file(ll_file, generate_diagram_script)
        if success:
            print("✓ SUCCESS")
            success_count += 1
        else:
            print("✗ FAILED")
            if error_msg:
                print(f"  Error: {error_msg}")
            failure_count += 1

    print("=" * 60)
    print(f"Results: {success_count} succeeded, {failure_count} failed")

    if failure_count > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
