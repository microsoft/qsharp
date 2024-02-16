#!/usr/bin/env python3

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import argparse
import os
import platform
import sys
import time
import venv
import shutil
import subprocess
import functools

from prereqs import check_prereqs

# Disable buffered output so that the log statements and subprocess output get interleaved in proper order
print = functools.partial(print, flush=True)

parser = argparse.ArgumentParser(
    description="Builds all projects in the repo, unless specific projects to build are passed "
    "as options, in which case only those projects are built."
)

parser.add_argument(
    "--cli", action="store_true", help="Build the command-line compiler"
)
parser.add_argument("--pip", action="store_true", help="Build the pip wheel")
parser.add_argument("--widgets", action="store_true", help="Build the Python widgets")
parser.add_argument("--wasm", action="store_true", help="Build the WebAssembly files")
parser.add_argument("--npm", action="store_true", help="Build the npm package")
parser.add_argument("--play", action="store_true", help="Build the web playground")
parser.add_argument("--samples", action="store_true", help="Compile the Q# samples")
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

parser.add_argument(
    "--check-prereqs",
    action=argparse.BooleanOptionalAction,
    default=True,
    help="Run the prerequisites check (default is --check-prereqs)",
)

parser.add_argument(
    "--integration-tests",
    action=argparse.BooleanOptionalAction,
    default=False,
    help="Build and run the integration tests (default is --no-integration-tests)",
)

args = parser.parse_args()

if args.check_prereqs:
    check_prereqs()

# If no specific project given then build all
build_all = (
    not args.cli
    and not args.pip
    and not args.widgets
    and not args.wasm
    and not args.samples
    and not args.npm
    and not args.play
    and not args.vscode
    and not args.jupyterlab
)
build_cli = build_all or args.cli
build_pip = build_all or args.pip
build_widgets = build_all or args.widgets
build_wasm = build_all or args.wasm
build_samples = build_all or args.samples
build_npm = build_all or args.npm
build_play = build_all or args.play
build_vscode = build_all or args.vscode
build_jupyterlab = build_all or args.jupyterlab

# JavaScript projects and eslint, prettier depend on npm_install
# However the JupyterLab extension uses yarn in a separate workspace
npm_install_needed = (
    build_npm or build_play or build_vscode or build_jupyterlab or args.check
)
npm_cmd = "npm.cmd" if platform.system() == "Windows" else "npm"

build_type = "debug" if args.debug else "release"
run_tests = args.test

root_dir = os.path.dirname(os.path.abspath(__file__))
wasm_src = os.path.join(root_dir, "wasm")
wasm_bld = os.path.join(root_dir, "target", "wasm32", build_type)
samples_src = os.path.join(root_dir, "samples")
npm_src = os.path.join(root_dir, "npm")
play_src = os.path.join(root_dir, "playground")
pip_src = os.path.join(root_dir, "pip")
widgets_src = os.path.join(root_dir, "widgets")
wheels_dir = os.path.join(root_dir, "target", "wheels")
vscode_src = os.path.join(root_dir, "vscode")
jupyterlab_src = os.path.join(root_dir, "jupyterlab")


def step_start(description):
    global start_time
    prefix = "::group::" if os.getenv("GITHUB_ACTIONS") == "true" else ""
    print(f"{prefix}build.py: {description}")
    start_time = time.time()


def step_end():
    global start_time
    duration = time.time() - start_time
    print(f"build.py: Finished in {duration:.3f}s.")
    if os.getenv("GITHUB_ACTIONS") == "true":
        print(f"::endgroup::")

def use_python_env(folder):
    # Check if in a virtual environment
    if (
        os.environ.get("VIRTUAL_ENV") is None
        and os.environ.get("CONDA_PREFIX") is None
        and os.environ.get("CI") is None
    ):
        print("Not in a virtual python environment")

        venv_dir = os.path.join(folder, ".venv")
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

    return python_bin


if npm_install_needed:
    step_start("Running npm install")
    subprocess.run([npm_cmd, "install"], check=True, text=True, cwd=root_dir)
    step_end()

if args.check:
    step_start("Running eslint and prettier checks")
    subprocess.run([npm_cmd, "run", "check"], check=True, text=True, cwd=root_dir)

    if build_wasm or build_cli:
        # If we're going to check the Rust code, do this before we try to compile it
        print("Running the cargo fmt and clippy checks")
        subprocess.run(
            ["cargo", "fmt", "--all", "--", "--check"],
            check=True,
            text=True,
            cwd=root_dir,
        )
        subprocess.run(
            [
                "cargo",
                "clippy",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ],
            check=True,
            text=True,
            cwd=root_dir,
        )
    step_end()

if build_cli:
    step_start("Building the command line compiler")
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
    step_end()

if build_pip:
    step_start("Building the pip package")

    python_bin = use_python_env(pip_src)

    # copy the process env vars
    pip_env: dict[str, str] = os.environ.copy()
    if platform.system() == "Darwin":
        # if on mac, add the arch flags for universal binary
        pip_env["ARCHFLAGS"] = "-arch x86_64 -arch arm64"

    pip_build_args = [
        python_bin,
        "-m",
        "pip",
        "wheel",
        "--wheel-dir",
        wheels_dir,
        pip_src,
    ]
    subprocess.run(pip_build_args, check=True, text=True, cwd=pip_src, env=pip_env)

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
        pip_install_args = [
            python_bin,
            "-m",
            "pip",
            "install",
            "--force-reinstall",
            "--no-index",
            "--find-links=" + wheels_dir,
            f"qsharp",
        ]
        subprocess.run(pip_install_args, check=True, text=True, cwd=pip_src)
        pytest_args = [python_bin, "-m", "pytest"]
        subprocess.run(
            pytest_args, check=True, text=True, cwd=os.path.join(pip_src, "tests")
        )

        qir_test_dir = os.path.join(pip_src, "tests-qir")
        # Try to install PyQIR and if successful, run additional tests.
        qir_install_args = [
            python_bin,
            "-m",
            "pip",
            "install",
            "-r",
            "test_requirements.txt",
        ]
        subprocess.run(qir_install_args, check=True, text=True, cwd=qir_test_dir)
        pyqir_check_args = [python_bin, "-c", "import pyqir"]
        if (
            subprocess.run(
                pyqir_check_args, check=False, text=True, cwd=qir_test_dir
            ).returncode
            == 0
        ):
            print("Running tests for the pip package with PyQIR")
            pytest_args = [python_bin, "-m", "pytest"]
            subprocess.run(pytest_args, check=True, text=True, cwd=qir_test_dir)
        else:
            print("Could not import PyQIR, skipping tests")
    step_end()

if build_widgets:
    step_start("Building the Python widgets")

    widgets_build_args = [
        sys.executable,
        "-m",
        "pip",
        "wheel",
        "--no-deps",
        "--wheel-dir",
        wheels_dir,
        widgets_src,
    ]
    subprocess.run(widgets_build_args, check=True, text=True, cwd=widgets_src)

    step_end()

if build_wasm:
    step_start("Building the wasm crate")
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
    step_end()

if build_samples:
    step_start("Building qsharp samples")
    project_directories = [dir for dir in os.walk(samples_src) if "qsharp.json" in dir[2]]
    single_file_directories = [candidate for candidate in os.walk(samples_src) if all([not proj_dir[0] in candidate[0] for proj_dir in project_directories])]

    files = [
        os.path.join(dp, f)
        for dp, _, filenames in single_file_directories
        for f in filenames
        if os.path.splitext(f)[1] == ".qs"
    ]
    projects =  [
        os.path.join(dp, f)
        for dp, _, filenames in project_directories
        for f in filenames
        if f == "qsharp.json"
    ]
    cargo_args = ["cargo", "run", "--bin", "qsc"]
    if build_type == "release":
        cargo_args.append("--release")
    for file in files:
        subprocess.run((cargo_args + ["--", file]), check=True, text=True, cwd=root_dir)
    for project in projects:
        subprocess.run((cargo_args + ["--", "--qsharp-json", project]), check=True, text=True, cwd=root_dir)
    step_end()

if build_npm:
    step_start("Building the npm package")
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
    step_end()

if build_play:
    step_start("Building the playground")
    play_args = [npm_cmd, "run", "build"]
    subprocess.run(play_args, check=True, text=True, cwd=play_src)
    step_end()

if build_vscode:
    step_start("Building the VS Code extension")
    vscode_args = [npm_cmd, "run", "build"]
    subprocess.run(vscode_args, check=True, text=True, cwd=vscode_src)
    step_end()
    if args.integration_tests:
        step_start("Running the VS Code integration tests")
        vscode_args = [npm_cmd, "test"]
        subprocess.run(vscode_args, check=True, text=True, cwd=vscode_src)
        step_end()


if build_jupyterlab:
    step_start("Building the JupyterLab extension")

    python_bin = use_python_env(jupyterlab_src)

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
    step_end()

if build_pip and build_widgets and args.integration_tests:
    step_start("Running notebook samples integration tests")
    # Find all notebooks in the samples directory. Skip the basic sample and the azure submission sample, since those won't run
    # nicely in automation.
    notebook_files = [os.path.join(dp, f) for dp, _, filenames in os.walk(samples_src) for f in filenames if f.endswith(".ipynb")
                    and not (f.startswith("sample.") or f.startswith("azure_submission."))]
    python_bin = use_python_env(samples_src)

    # copy the process env vars
    pip_env: dict[str, str] = os.environ.copy()

    # Install the qsharp package
    pip_install_args = [
        python_bin,
        "-m",
        "pip",
        "install",
        "--force-reinstall",
        "--no-index",
        "--find-links=" + wheels_dir,
        f"qsharp",
    ]
    subprocess.run(pip_install_args, check=True, text=True, cwd=pip_src)

    # Install the widgets package from the folder so dependencies are installed properly
    pip_install_args = [
        python_bin,
        "-m",
        "pip",
        "install",
        widgets_src,
    ]
    subprocess.run(pip_install_args, check=True, text=True, cwd=widgets_src, env=pip_env)

    # Install other dependencies
    pip_install_args = [
        python_bin,
        "-m",
        "pip",
        "install",
        "ipykernel",
        "nbconvert",
        "pandas",
    ]
    subprocess.run(pip_install_args, check=True, text=True, cwd=root_dir, env=pip_env)

    for notebook in notebook_files:
        print(f"Running {notebook}")
        # Run the notebook process, capturing stdout and only displaying it if there is an error
        result = subprocess.run([python_bin,
                        "-m",
                        "nbconvert",
                        "--to",
                        "notebook",
                        "--stdout",
                        "--ExecutePreprocessor.timeout=60",
                        "--sanitize-html",
                        "--execute",
                        notebook],
                        check=False, text=True, cwd=root_dir, env=pip_env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, encoding="utf-8")
        if result.returncode != 0:
            print(result.stdout)
            raise Exception(f"Error running {notebook}")

    step_end()
