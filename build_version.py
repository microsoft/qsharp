#!/usr/bin/env python3

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os
from datetime import datetime, timezone

# To be updated every time we start a new major.minor version.
major_minor = "1.0"

# Default to 'dev' builds with a YYMMDD build number
BUILD_TYPE = os.environ.get("BUILD_TYPE") or "dev"
BUILD_NUMBER = os.environ.get("BUILD_NUMBER") or "YYMMDD"

build_ver = datetime.now(timezone.utc).strftime("%y%m%d")

if BUILD_TYPE not in ["dev", "rc", "stable"]:
    print("BUILD_TYPE environment variable must be 'dev', 'rc', or 'stable'")
    exit(1)

print("Build type: {}".format(BUILD_TYPE))

# Override the dev build number if provided as an environment variable.
if BUILD_TYPE == "dev" and BUILD_NUMBER != "YYMMDD":
    try:
        build_ver = int(BUILD_NUMBER)
    except:
        print("BUILD_NUMBER environment variable is not valid")
        exit(1)

# Default to the dev build formats

# Python dev version of the format "2.3.0.dev1440"
pip_version = "{}.0.dev{}".format(major_minor, build_ver)

# Npm dev version of the format "2.3.0-dev.1440"
npm_version = "{}.0-dev.{}".format(major_minor, build_ver)

# VS Code dev version of the format "2.3.1440" (they don't support pre-release versions)
vscode_version = "{}.{}".format(major_minor, build_ver)

# Check if the environment variable BUILD_TYPE is set to 'rc' or 'stable'.
# If it is, we should only use a provided build number, not a default build number.
if BUILD_TYPE in ["rc", "stable"]:
    try:
        # Ensure build number is an integer.
        build_ver = int(BUILD_NUMBER)
    except:
        print("BUILD_NUMBER environment variable is not set to an integer")
        exit(1)

    # Python rc/stable version of the format "2.3.1rc0" or "2.3.1"
    pip_version = "{}.{}{}".format(
        major_minor, build_ver, "" if BUILD_TYPE == "stable" else "rc0"
    )

    # Npm rc/stable version of the format "2.3.1-rc" or "2.3.1"
    npm_version = "{}.{}{}".format(
        major_minor, build_ver, "" if BUILD_TYPE == "stable" else "-rc"
    )

    # VS Code rc/stable version of the format "2.3.1" (same for rc or stable)
    vscode_version = "{}.{}".format(major_minor, build_ver)

print("Pip version: {}".format(pip_version))
print("Npm version: {}".format(npm_version))
print("VS Code version: {}".format(vscode_version))


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

update_file(
    os.path.join(root_dir, "jupyterlab/pyproject.toml"),
    r'version = "0.0.0"',
    r'version = "{}"'.format(pip_version),
)

update_file(
    os.path.join(root_dir, "npm/package.json"),
    r'"version": "0.0.0",',
    r'"version": "{}",'.format(npm_version),
)

update_file(
    os.path.join(root_dir, "vscode/package.json"),
    r'"version": "0.0.0",',
    r'"version": "{}",'.format(vscode_version),
)

# If not a 'dev' build, update the VS Code extension identifier to be the non-dev version
if BUILD_TYPE != "dev":
    update_file(
        os.path.join(root_dir, "vscode/package.json"),
        r'"name": "qsharp-lang-vscode-dev",',
        r'"name": "qsharp-lang-vscode",',
    )
    # TODO: Update the description and/or readme also for the different extension channels
