#!/bin/bash

# Is to be executed in the Manylinux Docker container.
# https://github.com/pypa/manylinux

# Generates the manylinux .whl file.
# Input:
#   "/io" is mapped to this repo dir "qsharp" (from YAML, from repo dir  "docker run -v `pwd`:/io").
#   Environment (from YAML `docker run --env ARCH=x86_64`):
#       ARCH: Architecture: x86_64 or aarch64, etc.
# Output:
#   "<this repo>/target/wheels/qsharp-preview-*-cp3*-abi3-manylinux*_$ARCH.whl", 
#       e.g., "<this repo>/target/wheels/qsharp-preview-0.0.7-cp37-abi3-manylinux_2_28_x86_64.whl".

set -x -e

# Install Rust:
curl --proto '=https' --tlsv1.2 -sS https://sh.rustup.rs > rustup-init.sh
chmod +x rustup-init.sh
./rustup-init.sh -y
source "$HOME/.cargo/env"

# Enter this repo dir:
cd /io

# Build the platform-dependent Linux .whl:
/opt/python/cp311-cp311/bin/python ./build.py --pip --no-check --no-test --no-check-prereqs

# Make the generated Linux .whl a manylinux .whl:
export WHEEL_DIR_APATH=/io/target/wheels
export WHEEL_FILE_APATH=$WHEEL_DIR_APATH/qsharp_preview-*-cp37-abi3-linux_$ARCH.whl
auditwheel repair --wheel-dir $WHEEL_DIR_APATH $WHEEL_FILE_APATH
# The result is in, for example, "<this repo>/target/wheels/qsharp-preview-x.y.z-cp37-abi3-manylinux_2_28_x86_64.whl".
rm -fR $WHEEL_FILE_APATH    # Remove the platform-dependent Linux .whl (manylinux one remains).

exit  # From the Docker container
