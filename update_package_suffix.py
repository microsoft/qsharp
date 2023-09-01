#!/usr/bin/env python3

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import sys
import re
import os

# run with: python3 update_package_suffix.py <suffix>
# e.g. python3 update_package_suffix.py preview
# e.g. python3 update_package_suffix.py stable => remove suffix for stable

if len(sys.argv) < 2:
    # no name, leave as is
    sys.exit(0)

scriptDir = os.path.dirname(os.path.abspath(__file__))

suffix = sys.argv[1]
print(f"New package suffix: {suffix}")
if suffix == "stable":
    suffix = ""

for fileRPath in [
    os.path.join(scriptDir, "pip", "pyproject.toml"),  # name = "qsharp-preview"
    os.path.join(
        scriptDir, "jupyterlab", "package.json"
    ),  # "name": "qsharp-jupyterlab",
    os.path.join(
        scriptDir, "jupyterlab", "pyproject.toml"
    ),  # name = "qsharp-jupyterlab"
    os.path.join(scriptDir, "npm", "package.json"),  # "name": "qsharp",
    os.path.join(scriptDir, "vscode", "package.json"),  # "name": "qsharp-vscode",
]:
    print(fileRPath)

    regexp = r'^((?:name|jupyterlab)\s*=\s*\[?")([a-zA-z-_]+)([-|_])(nightly)("\]?\s*$)'

    if fileRPath.endswith("json"):
        regexp = r'(\s*\"name\"\s*:\s*")([a-zA-z-_]+)([-|_])(nightly)("\s*,\s*$)'

    # Read file:
    with open(fileRPath, "r") as file:
        lines = file.readlines()

    # Replace the line:
    lineIndex = 0  # Zero-based.
    for line in lines:
        match = re.match(regexp, line)
        if match:
            replacement = re.sub(regexp, r"\1\2\3" + suffix, line).rstrip(
                "_-"
            ) + re.sub(regexp, r"\5", line)
            lines[lineIndex] = replacement
            print(f"{lineIndex + 1}: {lines[lineIndex]}", end="")
        lineIndex = lineIndex + 1

    # Save file:
    with open(fileRPath, "w") as file:
        file.writelines(lines)
