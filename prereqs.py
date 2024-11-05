#!/usr/bin/env python3

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os
import urllib.request
import platform
import re
import sys
import subprocess
import tempfile
import functools

python_ver = (3, 11)  # Python support for Windows on ARM64 requires v3.11 or later
rust_ver = (1, 82)  # Ensure Rust version 1.82 or later is installed
node_ver = (
    18,
    17,
)
wasmpack_ver = (0, 12, 1)  # Latest tested wasm-pack version
rust_fmt_ver = (1, 7, 1)  # Current version when Rust 1.82 shipped
clippy_ver = (0, 1, 82)

# Disable buffered output so that the log statements and subprocess output get interleaved in proper order
print = functools.partial(print, flush=True)


def get_installed_rust_targets() -> str:
    try:
        args = ["rustup", "target", "list", "--installed"]
        return subprocess.check_output(args, universal_newlines=True)
    except subprocess.CalledProcessError as e:
        message = f"Unable to determine installed rust targets: {str(e)}"
        raise Exception(message)


def check_prereqs(install=False, skip_wasm=False):
    ### Check the Python version ###
    if (
        sys.version_info.major != python_ver[0]
        or sys.version_info.minor < python_ver[1]
    ):
        print(
            f"Python {python_ver[0]}.{python_ver[1]} or later is required. Please update"
        )
        exit(1)

    ### Check the Node.js version ###
    try:
        node_version = subprocess.check_output(["node", "-v"])
        print(f"Detected node.js version {node_version.decode()}")
    except FileNotFoundError:
        print("Node.js not found. Please install from https://nodejs.org/")
        exit(1)

    version_match = re.search(r"v(\d+)\.(\d+)\.\d+", node_version.decode())
    if version_match:
        node_major = int(version_match.group(1))
        node_minor = int(version_match.group(2))
        if node_major < node_ver[0] or (
            node_major == node_ver[0] and node_minor < node_ver[1]
        ):
            print(
                f"Node.js v{node_ver[0]}.{node_ver[1]} or later is required. Please update."
            )
            exit(1)
    else:
        raise Exception("Unable to determine the Node.js version.")

    ### Check the rustc compiler version ###
    try:
        rust_version = subprocess.check_output(["rustc", "--version"])
        print(f"Detected Rust version: {rust_version.decode()}")
    except FileNotFoundError:
        print("Rust compiler not found. Install from https://rustup.rs/")
        exit(1)

    version_match = re.search(r"rustc (\d+)\.(\d+).\d+", rust_version.decode())
    if version_match:
        rust_major = int(version_match.group(1))
        rust_minor = int(version_match.group(2))
        if rust_major < rust_ver[0] or (
            rust_major == rust_ver[0] and rust_minor < rust_ver[1]
        ):
            print(
                f'Rust v{rust_ver[0]}.{rust_ver[1]} or later is required. Please update with "rustup update"'
            )
            exit(1)
    else:
        raise Exception("Unable to determine the Rust compiler version.")

    ### Check the rustfmt version ###
    try:
        rust_fmt_version = subprocess.check_output(["cargo", "fmt", "--version"])
        print(f"Detected cargo fmt version: {rust_fmt_version.decode()}")
    except FileNotFoundError:
        print("cargo fmt not found. Install via rustup component add rustfmt")
        exit(1)

    version_match = re.search(r"rustfmt (\d+)\.(\d+)\.(\d+)", rust_fmt_version.decode())
    if version_match:
        rustfmt_major = int(version_match.group(1))
        rustfmt_minor = int(version_match.group(2))
        if rustfmt_major < rust_fmt_ver[0] or (
            rustfmt_major == rust_fmt_ver[0] and rustfmt_minor < rust_fmt_ver[1]
        ):
            print(
                f"cargo fmt v{rust_fmt_ver[0]}.{rust_fmt_ver[1]} or later is required. Please update"
            )
            exit(1)
    else:
        raise Exception("Unable to determine the rustfmt version")

    ### Check the clippy version ###
    try:
        clippy_version = subprocess.check_output(["cargo", "clippy", "--version"])
        print(f"Detected cargo clippy version: {clippy_version.decode()}")
    except FileNotFoundError:
        print("cargo clippy not found. Install via rustup component add clippy")
        exit(1)

    version_match = re.search(r"clippy (\d+)\.(\d+)\.(\d+)", clippy_version.decode())
    if version_match:
        clippy_major = int(version_match.group(1))
        clippy_minor = int(version_match.group(2))
        clippy_patch = int(version_match.group(3))
        if (
            clippy_major < clippy_ver[0]
            or (clippy_major == clippy_ver[0] and clippy_minor < clippy_ver[1])
            or (
                clippy_major == clippy_ver[0]
                and clippy_minor == clippy_ver[1]
                and clippy_patch < clippy_ver[2]
            )
        ):
            print(
                f"clippy v{clippy_ver[0]}.{clippy_ver[1]}.{clippy_ver[2]} or later is required. Please update"
            )
            exit(1)
    else:
        raise Exception("Unable to determine the clippy version")

    installed_rust_targets = get_installed_rust_targets()

    # On MacOS, ensure the required targets are installed
    if platform.system() == "Darwin":
        targets = ["aarch64-apple-darwin", "x86_64-apple-darwin"]
        if not all(target in installed_rust_targets for target in targets):
            print("One or both rust targets are not installed.")
            print("Please install the missing targets by running:")
            print("rustup target add aarch64-apple-darwin")
            print("rustup target add x86_64-apple-darwin")

    if not skip_wasm:
        wasm_checks(install, installed_rust_targets)


def wasm_checks(install, installed_rust_targets):
    ### Check the wasm_pack version ###
    try:
        wasm_pack_version = subprocess.check_output(["wasm-pack", "--version"])
        print(f"Detected wasm-pack version {wasm_pack_version.decode()}")
    except FileNotFoundError:
        if install == True:
            if platform.system() == "Windows":
                ver_str = f"v{wasmpack_ver[0]}.{wasmpack_ver[1]}.{wasmpack_ver[2]}"
                with urllib.request.urlopen(
                    f"https://github.com/rustwasm/wasm-pack/releases/download/{ver_str}/wasm-pack-init.exe"
                ) as wasm_exe:
                    exe_bytes = wasm_exe.read()
                    tmp_dir = os.getenv("RUNNER_TEMP", default=tempfile.gettempdir())
                    file_name = os.path.join(tmp_dir, "wasm-pack-init.exe")
                    with open(file_name, "wb") as exe_file:
                        exe_file.write(exe_bytes)
                    print("Attempting to install wasm-pack")
                    subprocess.run([file_name, "/q"], check=True)
            else:
                with urllib.request.urlopen(
                    "https://rustwasm.github.io/wasm-pack/installer/init.sh"
                ) as wasm_script:
                    sh_text = wasm_script.read().decode("utf-8")
                    tmp_dir = os.getenv("RUNNER_TEMP", default=tempfile.gettempdir())
                    file_name = os.path.join(tmp_dir, "wasm_install.sh")
                    with open(file_name, "w") as file:
                        file.write(sh_text)
                    print("Attempting to install wasm-pack")
                    subprocess.run(["sh", file_name], check=True)

            wasm_pack_version = subprocess.check_output(["wasm-pack", "--version"])
        else:
            print(
                "wasm-pack not found. Please install from https://rustwasm.github.io/wasm-pack/installer/"
            )
            exit(1)

    version_match = re.search(r"wasm-pack (\d+)\.(\d+).\d+", wasm_pack_version.decode())
    if version_match:
        wasm_major = int(version_match.group(1))
        wasm_minor = int(version_match.group(2))
        if wasm_major != wasmpack_ver[0] or wasm_minor < wasmpack_ver[1]:
            print(
                f"wasm-pack version must be {wasmpack_ver[0]}.{wasmpack_ver[1]} or later. Please update."
            )
            exit(1)
    else:
        print("Unable to determine the wasm-pack version")

    # Ensure the required wasm target is installed
    if "wasm32-unknown-unknown" not in installed_rust_targets:
        print("WASM rust target is not installed.")
        print("Please install the missing target by running:")
        print("rustup target add wasm32-unknown-unknown")


if __name__ == "__main__":
    skip_wasm = "--skip-wasm" in sys.argv
    install = "--install" in sys.argv
    check_prereqs(install=install, skip_wasm=skip_wasm)
