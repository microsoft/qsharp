#!/usr/bin/env python3

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import argparse
import os
import platform
import re
import sys
import venv
import shutil
import subprocess

from prereqs import check_prereqs

parser = argparse.ArgumentParser(
    description="Builds all projects in the repo, unless specific projects to build are passed "
    "as options, in which case only those projects are built."
)

parser.add_argument(
    "--cli", action="store_true", help="Build the command-line compiler"
)
parser.add_argument("--pip", action="store_true", help="Build the pip wheel")
parser.add_argument("--wasm", action="store_true", help="Build the WebAssembly files")
parser.add_argument("--npm", action="store_true", help="Build the npm package")
parser.add_argument("--play", action="store_true", help="Build the web playground")
parser.add_argument("--vscode", action="store_true", help="Build the VS Code extension")
parser.add_argument(
    "--jupyterlab", action="store_true", help="Build the JupyterLab extension"
)

parser.add_argument(
    "--debug", action="store_true", help="Create a debug build (default is release)"
)
parser.add_argument(
    "--test",
    action=argparse.BooleanOptionalAction,
    default=True,
    help="Run the tests (default is --test)",
)

# Below allows for passing --no-check to avoid the default of True
parser.add_argument(
    "--check",
    action=argparse.BooleanOptionalAction,
    default=True,
    help="Run the linting and formatting checks (default is --check)",
)

args = parser.parse_args()

check_prereqs()

# If no specific project given then build all
build_all = (
    not args.cli
    and not args.pip
    and not args.wasm
    and not args.npm
    and not args.play
    and not args.vscode
    and not args.jupyterlab
)
build_cli = build_all or args.cli
build_pip = build_all or args.pip
build_wasm = build_all or args.wasm
build_npm = build_all or args.npm
build_play = build_all or args.play
build_vscode = build_all or args.vscode
build_jupyterlab = build_all or args.jupyterlab

npm_install_needed = build_npm or build_play or build_vscode or build_jupyterlab
npm_cmd = "npm.cmd" if platform.system() == "Windows" else "npm"

build_type = "debug" if args.debug else "release"
run_tests = args.test

root_dir = os.path.dirname(os.path.abspath(__file__))
wasm_src = os.path.join(root_dir, "wasm")
wasm_bld = os.path.join(root_dir, "target", "wasm32", build_type)
npm_src = os.path.join(root_dir, "npm")
play_src = os.path.join(root_dir, "playground")
pip_src = os.path.join(root_dir, "pip")
wheels_dir = os.path.join(root_dir, "target", "wheels")
vscode_src = os.path.join(root_dir, "vscode")
jupyterlab_src = os.path.join(root_dir, "jupyterlab")

if npm_install_needed:
    subprocess.run([npm_cmd, "install"], check=True, text=True, cwd=root_dir)

    # The projects that require npm are also the ones we want to check (which depends on npm also)
    if args.check:
        print("Running eslint, prettier, and tsc checks")
        subprocess.run([npm_cmd, "run", "check"], check=True, text=True, cwd=root_dir)

# If we're going to check the Rust code, do this before we try to compile it
if args.check and (build_wasm or build_cli):
    print("Running the cargo fmt and clippy checks")
    subprocess.run(
        ["cargo", "fmt", "--all", "--", "--check"], check=True, text=True, cwd=root_dir
    )
    subprocess.run(
        ["cargo", "clippy", "--all-targets", "--all-features", "--", "-D", "warnings"],
        check=True,
        text=True,
        cwd=root_dir,
    )

if build_cli:
    print("Building the command line compiler")
    cargo_build_args = ["cargo", "build"]
    if build_type == "release":
        cargo_build_args.append("--release")
    subprocess.run(cargo_build_args, check=True, text=True, cwd=root_dir)

    if run_tests:
        print("Running tests for the command line compiler")
        cargo_test_args = ["cargo", "test"]
        if build_type == "release":
            cargo_test_args.append("--release")
        subprocess.run(cargo_test_args, check=True, text=True, cwd=root_dir)

if build_pip:
    print("Building the pip package")
    # Check if in a virtual environment
    if (
        os.environ.get("VIRTUAL_ENV") is None
        and os.environ.get("CONDA_PREFIX") is None
        and os.environ.get("CI") is None
    ):
        print("Not in a virtual python environment")

        venv_dir = os.path.join(pip_src, ".venv")
        # Create virtual environment under repo root
        if not os.path.exists(venv_dir):
            print(f"Creating a virtual environment under {venv_dir}")
            venv.main([venv_dir])

        # Check if .venv/bin/python exists, otherwise use .venv/Scripts/python.exe (Windows)
        python_bin = os.path.join(venv_dir, "bin", "python")
        if not os.path.exists(python_bin):
            python_bin = os.path.join(venv_dir, "Scripts", "python.exe")
        print(f"Using python from {python_bin}")
    else:
        # Already in a virtual environment, use current Python
        python_bin = sys.executable

    pip_build_args = [
        python_bin,
        "-m",
        "pip",
        "wheel",
        "--wheel-dir",
        wheels_dir,
        pip_src,
    ]
    subprocess.run(pip_build_args, check=True, text=True, cwd=pip_src)

    if run_tests:
        print("Running tests for the pip package")

        pip_install_args = [
            python_bin,
            "-m",
            "pip",
            "install",
            "-r",
            "test_requirements.txt",
        ]
        subprocess.run(pip_install_args, check=True, text=True, cwd=pip_src)
        pip_install_args = [python_bin, "-m", "pip", "install", "-e", "."]
        subprocess.run(pip_install_args, check=True, text=True, cwd=pip_src)
        pytest_args = [python_bin, "-m", "pytest"]
        subprocess.run(pytest_args, check=True, text=True, cwd=pip_src)

if build_wasm:
    print("Building the wasm crate")
    # wasm-pack can't build for web and node in the same build, so need to run twice.
    # Hopefully not needed if https://github.com/rustwasm/wasm-pack/issues/313 lands.
    build_flag = "--release" if build_type == "release" else "--dev"

    wasm_pack_args = ["wasm-pack", "build", build_flag]
    web_build_args = ["--target", "web", "--out-dir", os.path.join(wasm_bld, "web")]
    node_build_args = [
        "--target",
        "nodejs",
        "--out-dir",
        os.path.join(wasm_bld, "node"),
    ]
    subprocess.run(wasm_pack_args + web_build_args, check=True, text=True, cwd=wasm_src)
    subprocess.run(
        wasm_pack_args + node_build_args, check=True, text=True, cwd=wasm_src
    )

if build_npm:
    print("Building the npm package")
    # Copy the wasm build files over for web and node targets
    for target in ["web", "node"]:
        lib_dir = os.path.join(npm_src, "lib", target)
        os.makedirs(lib_dir, exist_ok=True)

        for filename in ["qsc_wasm_bg.wasm", "qsc_wasm.d.ts", "qsc_wasm.js"]:
            fullpath = os.path.join(wasm_bld, target, filename)

            # To make the node files CommonJS modules, the extension needs to change
            # (This is because the package is set to ECMAScript modules by default)
            if target == "node" and filename == "qsc_wasm.js":
                filename = "qsc_wasm.cjs"
            if target == "node" and filename == "qsc_wasm.d.ts":
                filename = "qsc_wasm.d.cts"

            shutil.copy2(fullpath, os.path.join(lib_dir, filename))

    npm_args = [npm_cmd, "run", "build"]
    subprocess.run(npm_args, check=True, text=True, cwd=npm_src)

    if run_tests:
        print("Running tests for the npm package")
        npm_test_args = ["node", "--test"]
        subprocess.run(npm_test_args, check=True, text=True, cwd=npm_src)

if build_play:
    print("Building the playground")
    play_args = [npm_cmd, "run", "build"]
    subprocess.run(play_args, check=True, text=True, cwd=play_src)

if build_vscode:
    print("Building the VS Code extension")
    vscode_args = [npm_cmd, "run", "build"]
    subprocess.run(vscode_args, check=True, text=True, cwd=vscode_src)

if build_jupyterlab:
    print("Building the JupyterLab extension")

    # Check if in a virtual environment
    if (
        os.environ.get("VIRTUAL_ENV") is None
        and os.environ.get("CONDA_PREFIX") is None
        and os.environ.get("CI") is None
    ):
        print("Not in a virtual python environment")

        venv_dir = os.path.join(jupyterlab_src, ".venv")
        # Create virtual environment under repo root
        if not os.path.exists(venv_dir):
            print(f"Creating a virtual environment under {venv_dir}")
            venv.main([venv_dir])

        # Check if .venv/bin/python exists, otherwise use .venv/Scripts/python.exe (Windows)
        python_bin = os.path.join(venv_dir, "bin", "python")
        if not os.path.exists(python_bin):
            python_bin = os.path.join(venv_dir, "Scripts", "python.exe")
        print(f"Using python from {python_bin}")
    else:
        # Already in a virtual environment, use current Python
        python_bin = sys.executable

    pip_build_args = [
        python_bin,
        "-m",
        "pip",
        "wheel",
        "--wheel-dir",
        wheels_dir,
        jupyterlab_src,
    ]
    subprocess.run(pip_build_args, check=True, text=True, cwd=jupyterlab_src)

    if run_tests:
        print("Running tests for the JupyterLab extension")

        pip_install_args = [
            python_bin,
            "-m",
            "pip",
            "install",
            "-r",
            "test_requirements.txt",
        ]
        subprocess.run(pip_install_args, check=True, text=True, cwd=jupyterlab_src)
        pip_install_args = [python_bin, "-m", "pip", "install", "-e", "."]
        subprocess.run(pip_install_args, check=True, text=True, cwd=jupyterlab_src)
        pytest_args = [python_bin, "-m", "pytest"]
        subprocess.run(pytest_args, check=True, text=True, cwd=jupyterlab_src)
