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

def check_prereqs(install=False):
    # Python support for Windows on ARM64 requires v3.11 or later
    if sys.version_info.major != 3 or sys.version_info.minor < 11:
        print('Python 3.11 or later is required to support all target platforms.')
        exit(1)

    # Ensure Rust version 1.65 or later is installed (needed for 'backtrace' support)
    try:
        rust_version = subprocess.check_output(['rustc', '--version'])
        print(f"Detected Rust version: {rust_version.decode()}")
    except FileNotFoundError:
        print('Rust compiler version 1.65 or later is required. Install from https://rustup.rs/')
        exit(1)

    version_match = re.search(r'rustc (\d+)\.(\d+).\d+', rust_version.decode())
    if version_match:
        rust_major = int(version_match.group(1))
        rust_minor = int(version_match.group(2))
        if rust_major < 1 or rust_major == 1 and rust_minor < 65:
            print('Rust v1.65 or later is required. Please update with "rustup update"')
            exit(1)
    else:
        raise Exception('Unable to determine the Rust compiler version.')

    # Node.js version 16.17 or later is required to support the Node.js 'test' module
    try:
        node_version = subprocess.check_output(['node', '-v'])
        print(f"Detected node.js version {node_version.decode()}")
    except FileNotFoundError:
        print('Node.js v16.17 or later is required. Please install from https://nodejs.org/')
        exit(1)

    version_match = re.search(r'v(\d+)\.(\d+)\.\d+', node_version.decode())
    if version_match:
        node_major = int(version_match.group(1))
        node_minor = int(version_match.group(2))
        if node_major < 16 or node_major == 16 and node_minor < 17:
            print('Node.js version must be 16.17.0 or later. Please update.')
            exit(1)
    else:
        raise Exception('Unable to determine the Node.js version.')

    # Check that wasm-pack v0.10 or later is installed
    try:
        wasm_pack_version = subprocess.check_output(['wasm-pack', '--version'])
        print(f"Detected wasm-pack version {wasm_pack_version.decode()}")
    except FileNotFoundError:
        if install == True:
            if platform.system() == 'Windows':
                with urllib.request.urlopen('https://github.com/rustwasm/wasm-pack/releases/download/v0.11.0/wasm-pack-init.exe') as wasm_exe:
                    exe_bytes = wasm_exe.read()
                    tmp_dir = os.getenv('RUNNER_TEMP', default=tempfile.gettempdir())
                    file_name = os.path.join(tmp_dir, 'wasm-pack-init.exe')
                    with open(file_name, "wb") as exe_file:
                        exe_file.write(exe_bytes)
                    print('Attempting to install wasm-pack')
                    subprocess.run([file_name, '/q'], check=True)
            else:
                with urllib.request.urlopen('https://rustwasm.github.io/wasm-pack/installer/init.sh') as wasm_script:
                    sh_text = wasm_script.read().decode('utf-8')
                    tmp_dir = os.getenv('RUNNER_TEMP', default=tempfile.gettempdir())
                    file_name = os.path.join(tmp_dir, 'wasm_install.sh')
                    with open(file_name, "w") as file:
                        file.write(sh_text)
                    print('Attempting to install wasm-pack')
                    subprocess.run(['sh', file_name], check=True)

            wasm_pack_version = subprocess.check_output(['wasm-pack', '--version'])
        else:
            print('wasm-pack v0.10 or later is required. Please install from https://rustwasm.github.io/wasm-pack/installer/')
            exit(1)

    version_match = re.search(r'wasm-pack (\d+)\.(\d+).\d+', wasm_pack_version.decode())
    if version_match:
        wasm_major = int(version_match.group(1))
        wasm_minor = int(version_match.group(2))
        if wasm_major == 0 and wasm_minor < 10:
            print('wasm-pack version must be 0.10 or later. Please update.')
            exit(1)
    else:
        raise Exception('Unable to determine the wasm-pack version')

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == '--install':
        check_prereqs(install=True)
    else:
        check_prereqs(install=False)
