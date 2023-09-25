#!/usr/bin/env python3

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import argparse
import functools
import os
import platform
import subprocess
import time

from build import build

# Disable buffered output so that the log statements and subprocess output get interleaved in proper order
print = functools.partial(print, flush=True)

parser = argparse.ArgumentParser(
    description="Runs integration tests for the repo.",

)

parser.add_argument(
    "--build",
    action=argparse.BooleanOptionalAction,
    default=True,
    help="Build the repo first (default is --build). When this option is used, "
    "any unparsed command line arguments are passed directly to build.py."
)

parser.add_argument(
    "build_args",
    nargs="*",
    help="When --build is used, arguments passed to build.py. Example: `test.py --build -- --debug`"
)

args = parser.parse_args()

def step_start(description):
    global start_time
    print(f"test.py step: {description}")
    start_time = time.time()

def step_end():
    global start_time
    duration = time.time() - start_time
    print(f"test.py step: Finished in {duration:.3f}s.")


if args.build:
    step_start("Building repo")
    # Pass the rest of the arguments to build.py
    build(['--no-check', '--no-test', '--no-samples', *args.build_args])
    step_end()

npm_cmd = "npm.cmd" if platform.system() == "Windows" else "npm"

root_dir = os.path.dirname(os.path.abspath(__file__))
vscode_src = os.path.join(root_dir, "vscode")
    
step_start("Running the VS Code integration tests")
vscode_args = [npm_cmd, "test"]
subprocess.run(vscode_args, check=True, text=True, cwd=vscode_src)
step_end()
