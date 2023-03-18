#!/usr/bin/env python3

import argparse
import os
import subprocess

parser = argparse.ArgumentParser(description=
"Builds all projects in the repo, unless specific projects to build are passed "
"as options, in which case only those projects are built.")

parser.add_argument('--release', action='store_true',
                    help='Create a release build (default is debug)')

parser.add_argument('--cli', action='store_true',
                    help='Build the command-line compiler')
parser.add_argument('--wasm', action='store_true',
                    help='Build the WebAssembly files')
# TODO: Add '--test' option

args = parser.parse_args()

# If no specific project given then build all
build_all  = not args.cli and not args.wasm
build_cli  = build_all or args.cli
build_wasm = build_all or args.wasm

build_type = 'release' if args.release else 'debug'

root_dir = os.path.dirname(os.path.abspath(__file__))
wasm_src = os.path.join(root_dir, "compiler", "qsc_wasm")
wasm_bld = os.path.join(root_dir, 'target', 'wasm32', build_type)

if build_cli:
    cargo_build_args = ['cargo', 'build']
    if args.release:
        cargo_build_args.append('--release')
    result = subprocess.run(cargo_build_args, check=True,
                            text=True, cwd=root_dir)

if build_wasm:
    wasm_pack_args = ['wasm-pack', 'build',
                    '--target', 'web',
                    '--out-dir', wasm_bld,
                    ('--release' if args.release else '--dev'),
                    '--features', 'wasm']
    result = subprocess.run(wasm_pack_args, check=True,
                            text=True, cwd=wasm_src)
