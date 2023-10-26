#!/usr/bin/env bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

set -e

apt-get update

# install cross compiler toolchain
DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    g++-aarch64-linux-gnu

# install emulation support
DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    qemu qemu-system-misc qemu-user-static qemu-user
