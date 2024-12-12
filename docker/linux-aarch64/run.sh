#!/usr/bin/env bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo "SCRIPT_DIR: ${SCRIPT_DIR}"

BASE_IMAGE="${BASE_IMAGE:-mcr.microsoft.com/cbl-mariner/base/python:3.9}"
echo "BASE_IMAGE: ${BASE_IMAGE}"

VOLUME_ROOT=$(realpath ${SCRIPT_DIR}/../..)
echo "VOLUME_ROOT: ${VOLUME_ROOT}"

echo "docker run --platform linux/arm64/v8 -v ${VOLUME_ROOT}:/qsharp -e WHEEL_DIR='/qsharp/target/wheels' ${BASE_IMAGE} bash /qsharp/docker/linux-aarch64/entrypoint.sh"
docker run --platform linux/arm64/v8 -v ${VOLUME_ROOT}:/qsharp -e WHEEL_DIR='/qsharp/target/wheels' ${BASE_IMAGE} bash /qsharp/docker/linux-aarch64/entrypoint.sh
