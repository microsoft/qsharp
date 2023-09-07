#!/usr/bin/env python3

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os
from datetime import datetime, timezone

# The major.minor version of the build, and the date we started on this version.
# To be updated every time we start a new major.minor version.
major_minor = "1.0"
version_date = datetime(2023, 9, 6, tzinfo=timezone.utc)

# The 'dev' build number is the number of minutes since the version start date.
# This is to ensure that the build number is always increasing, even if we
# publish multiple dev builds in an hour (e.g. due to bugs or other issues).
now_time = datetime.now(timezone.utc)
build_ver = int((now_time - version_date).total_seconds() / 60)

#### TODO: Above is a bad idea for a matrix of builds, as each platform might get a different build number
# For now, just go with the date of the build for dev builds in yymmdd format.
build_ver = now_time.strftime("%y%m%d")

# Check if the environment variable BUILD_TYPE is set to 'rc' or 'stable'.
# If it is, we should use a provided build number instead of the dev build number.
BUILD_TYPE = os.environ.get("BUILD_TYPE") or "dev"
BUILD_NUMBER = os.environ.get("BUILD_NUMBER") or "YYMMDD"

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

# Python dev version something like "2.3.0.dev1440"
pip_version = "{}.0.dev{}".format(major_minor, build_ver)

# Npm dev version something like "2.3.0-dev.1440"
npm_version = "{}.0-dev.{}".format(major_minor, build_ver)

# VS Code dev version something like "2.3.1440" (they don't support pre-release versions)
vscode_version = "{}.{}".format(major_minor, build_ver)

# Update the rc or stable build formats if needed
if BUILD_TYPE in ["rc", "stable"]:
    try:
        # Ensure build number is an integer.
        build_ver = int(BUILD_NUMBER)
    except:
        print("BUILD_NUMBER environment variable is not set to an integer")
        exit(1)

    # Python rc/stable version something like "2.3.1rc0" or "2.3.1"
    pip_version = "{}.{}{}".format(
        major_minor, build_ver, "" if BUILD_TYPE == "stable" else "rc0"
    )

    # Npm rc/stable version something like "2.3.1-rc" or "2.3.1"
    npm_version = "{}.{}{}".format(
        major_minor, build_ver, "" if BUILD_TYPE == "stable" else "-rc"
    )

    # VS Code rc/stable version something like "2.3.1" (same for rc or stable)
    vscode_version = "{}.{}".format(major_minor, build_ver)

# Verify versions set as expected
print("Pip version: {}".format(pip_version))
print("Npm version: {}".format(npm_version))
print("VS Code version: {}".format(vscode_version))


def update_file(file, old_text, new_text):
    # Open the file and replace the first string matching the old text with the new text
    with open(file, "r+") as f:
        contents = f.read()
        new_contents = contents.replace(old_text, new_text, 1)
        f.seek(0)
        f.write(new_contents)
        f.truncate()


# TODO: Make paths absolute relative to the script location
update_file(
    "pip/pyproject.toml", r'version = "0.0.0"', r'version = "{}"'.format(pip_version)
)

update_file(
    "jupyterlab/pyproject.toml",
    r'version = "0.0.0"',
    r'version = "{}"'.format(pip_version),
)

update_file(
    "npm/package.json",
    r'"version": "0.0.0",',
    r'"version": "{}",'.format(npm_version),
)

update_file(
    "vscode/package.json",
    r'"version": "0.0.0",',
    r'"version": "{}",'.format(vscode_version),
)

# If not a 'dev' build, update the VS Code extension identifier to be the non-dev version
if BUILD_TYPE != "dev":
    update_file(
        "vscode/package.json",
        r'"name": "qsharp-lang-vscode-dev",',
        r'"name": "qsharp-lang-vscode",',
    )
    # TODO: Update the description and/or readme also for the different extension channels
