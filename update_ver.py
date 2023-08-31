#!/usr/bin/env python3

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import sys
import re
import os
from packaging.version import Version, parse

if len(sys.argv) < 2:
    print("Argument is missing. Please specify the new package version, e.g. 0.0.8")
    sys.exit(-1)

newVer = parse(sys.argv[1])
if not isinstance(newVer, Version):
    print("Argument not a valid version")
    sys.exit(-2)

# ensure that we have a 3-part version or rust will fail
newVerFormatted = f"{newVer.major}.{newVer.minor}.{newVer.micro}"

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
    replacement = f'version = "{newVerFormatted}"\n'
    if fileRPath.endswith("package.json"):
        regexp = (
            '\s*"version"\s*:\s*"\d+\.\d+\.\d+"\s*,\s*$'  # `  "version": "0.0.11",`
        )
        replacement = f'  "version": "{newVerFormatted}",\n'

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
