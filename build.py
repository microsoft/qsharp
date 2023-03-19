#!/usr/bin/env python3

import argparse
import os
import shutil
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
parser.add_argument('--npm', action='store_true',
                    help='Build the npm package')
parser.add_argument('--play', action='store_true',
                    help='Build the web playground')
# TODO: Add '--test' option

args = parser.parse_args()

# If no specific project given then build all
build_all  = not args.cli and not args.wasm and not args.npm and not args.play
build_cli  = build_all or args.cli
build_wasm = build_all or args.wasm
build_npm  = build_all or args.npm
build_play = build_all or args.play

build_type = 'release' if args.release else 'debug'

root_dir = os.path.dirname(os.path.abspath(__file__))
wasm_src = os.path.join(root_dir, "compiler", "qsc_wasm")
wasm_bld = os.path.join(root_dir, 'target', 'wasm32', build_type)
wasm_web_dir = os.path.join(wasm_bld, 'web')
wasm_node_dir = os.path.join(wasm_bld, 'node')
npm_src  = os.path.join(root_dir, "npm")
play_src = os.path.join(root_dir, "playground")

if build_cli:
    cargo_build_args = ['cargo', 'build']
    if args.release:
        cargo_build_args.append('--release')
    result = subprocess.run(cargo_build_args, check=True,
                            text=True, cwd=root_dir)

if build_wasm:
    # wasm-pack can't build for web and node in the same build, so need to run twice.
    # Hopefully not needed if https://github.com/rustwasm/wasm-pack/issues/313 lands.
    build_type = ('--release' if args.release else '--dev')
    cargo_options = ['--features', 'wasm']

    wasm_pack_args = ['wasm-pack', 'build', build_type]
    web_build_args = ['--target', 'web', '--out-dir', wasm_web_dir]
    node_build_args = ['--target', 'nodejs', '--out-dir', wasm_node_dir]
    subprocess.run(wasm_pack_args + web_build_args + cargo_options,
                   check=True, text=True, cwd=wasm_src)
    subprocess.run(wasm_pack_args + node_build_args + cargo_options,
                   check=True, text=True, cwd=wasm_src)

if build_npm:
    # Copy the wasm build files over for web and node targets
    for target in ['web', 'node']:
        lib_dir = os.path.join(npm_src, 'lib', target)
        os.makedirs(lib_dir, exist_ok = True)

        for filename in ['qsc_wasm_bg.wasm', 'qsc_wasm.d.ts', 'qsc_wasm.js']:
            fullpath = os.path.join(wasm_bld, target, filename)

            # To make the node files CommonJS modules, the extension needs to change
            # (This is because the package is set to ECMAScript modules by default)
            if target == 'node' and filename == 'qsc_wasm.js':
                filename = 'qsc_wasm.cjs'
            if target == 'node' and filename == 'qsc_wasm.d.ts':
                filename = 'qsc_wasm.d.cts'

            shutil.copy2(fullpath, os.path.join(lib_dir, filename))
    
    npm_args = ['npm', 'run', 'build']
    result = subprocess.run(npm_args, check=True, text=True, cwd=npm_src)

if build_play:
    play_args = ['npm', 'run', 'build']
    result = subprocess.run(play_args, check=True, text=True, cwd=play_src)
