#!/usr/bin/env python3

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os
from datetime import datetime, timezone

# To be updated every time we start a new major.minor version.
major_minor = "1.0"

# Default to 'dev' builds
BUILD_TYPE = os.environ.get("BUILD_TYPE") or "dev"
BUILD_NUMBER = os.environ.get("BUILD_NUMBER")

if BUILD_TYPE not in ["dev", "rc", "stable"]:
    print("BUILD_TYPE environment variable must be 'dev', 'rc', or 'stable'")
    exit(1)

try:
    build_ver = int(BUILD_NUMBER)
except:
    print("BUILD_NUMBER environment variable must be set to a valid integer")
    exit(1)

print("Build type: {}".format(BUILD_TYPE))

version_triple = "{}.{}".format(major_minor, build_ver)

pip_suffix = {"stable": "", "rc": "rc0", "dev": ".dev0"}
npm_suffix = {"stable": "", "rc": "-rc", "dev": "-dev"}

pip_version = "{}{}".format(version_triple, pip_suffix.get(BUILD_TYPE))
npm_version = "{}{}".format(version_triple, npm_suffix.get(BUILD_TYPE))

print("Pip version: {}".format(pip_version))
print("Npm version: {}".format(npm_version))
print("VS Code version: {}".format(version_triple))


def update_file(file, old_text, new_text):
    # Open the file and replace the first string matching the old text with the new text
    with open(file, "r+", newline="") as f:
        contents = f.read()
        new_contents = contents.replace(old_text, new_text, 1)
        f.seek(0)
        f.write(new_contents)
        f.truncate()


root_dir = os.path.dirname(os.path.abspath(__file__))
update_file(
    os.path.join(root_dir, "pip/pyproject.toml"),
    r'version = "0.0.0"',
    r'version = "{}"'.format(pip_version),
)

# Publish the jupyterlab extension without the 'pre-release' tagging for rc builds.
# It is already stable and the prior publishing (and yanking) of release versions causes issues.
update_file(
    os.path.join(root_dir, "jupyterlab/pyproject.toml"),
    r'version = "0.0.0"',
    r'version = "{}"'.format(pip_version if BUILD_TYPE == "dev" else version_triple),
)

update_file(
    os.path.join(root_dir, "npm/package.json"),
    r'"version": "0.0.0",',
    r'"version": "{}",'.format(npm_version),
)

update_file(
    os.path.join(root_dir, "vscode/package.json"),
    r'"version": "0.0.0",',
    r'"version": "{}",'.format(version_triple),
)

# If not a 'dev' build, update the VS Code extension identifier to be the non-dev version
if BUILD_TYPE != "dev":
    update_file(
        os.path.join(root_dir, "vscode/package.json"),
        r'"name": "qsharp-lang-vscode-dev",',
        r'"name": "qsharp-lang-vscode",',
    )
    update_file(
        os.path.join(root_dir, "vscode/package.json"),
        r"[DEV BUILD] Azure Quantum Development Kit",
        r"Azure Quantum Development Kit",
    )

else:
    # Update the README to contain the dev version contents
    with open(
        os.path.join(root_dir, "vscode/README-DEV.md"), "r", newline=""
    ) as dev_readme:
        contents = dev_readme.read()
    with open(os.path.join(root_dir, "vscode/README.md"), "w", newline="") as readme:
        readme.write(contents)
