#!/usr/bin/env python3

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import sys
import re
import os

if len(sys.argv) < 2:
    print("Argument is missing. Please specify the new package version, e.g. 0.0.8")
    sys.exit()

newVer = sys.argv[1]
scriptDir = os.path.dirname(os.path.abspath(__file__))

for fileRPath in [
    os.path.join(scriptDir, "Cargo.toml"),
    os.path.join(scriptDir, "pip", "pyproject.toml"),
    os.path.join(scriptDir, "jupyterlab", "package.json"),
    os.path.join(scriptDir, "npm", "package.json"),
    os.path.join(scriptDir, "playground", "package.json"),
    os.path.join(scriptDir, "vscode", "package.json"),
]:
    print(fileRPath)

    # Config:
    regexp = '^version\s*=\s*"\d+\.\d+\.\d+"\s*$'  # `version = "0.0.11"`
    replacement = f'version = "{newVer}"\n'
    if fileRPath.endswith("package.json"):
        regexp = (
            '\s*"version"\s*:\s*"\d+\.\d+\.\d+"\s*,\s*$'  # `  "version": "0.0.11",`
        )
        replacement = f'  "version": "{newVer}",\n'

    # Read file:
    with open(fileRPath, "r") as file:
        lines = file.readlines()

    # Replace the line:
    lineIndex = 0  # Zero-based.
    for line in lines:
        if re.match(regexp, line):
            lines[lineIndex] = replacement
            print(f"{lineIndex + 1}: {lines[lineIndex]}", end="")
            break
        lineIndex = lineIndex + 1

    # Save file:
    with open(fileRPath, "w") as file:
        file.writelines(lines)
