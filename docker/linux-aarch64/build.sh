#!/usr/bin/env bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo "SCRIPT_DIR: ${SCRIPT_DIR}"

BASE_IMAGE="${BASE_IMAGE:-ubuntu:20.04}"
echo "BASE_IMAGE: ${BASE_IMAGE}"

TAG="${TAG:-qsharp-lang-linux-aarch64-runner}"
echo "TAG: ${TAG}"

docker build -t ${TAG} --build-arg BASE_IMAGE=${BASE_IMAGE} -f ${SCRIPT_DIR}/Dockerfile ${SCRIPT_DIR}
