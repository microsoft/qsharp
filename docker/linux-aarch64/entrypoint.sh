#!/usr/bin/env bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

set -e

alias python=python3

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo "SCRIPT_DIR: ${SCRIPT_DIR}"

WHEEL_ARCH="${WHEEL_ARCH:-aarch64}"
echo "WHEEL_ARCH: ${WHEEL_ARCH}"

WHEEL_PLATFORM="${WHEEL_PLATFORM:-manylinux_2_31_${WHEEL_ARCH}}"
echo "WHEEL_PLATFORM: ${WHEEL_PLATFORM}"

PIP_DIR="${PIP_DIR:-${SCRIPT_DIR}/../../pip}"
echo "PIP_DIR: ${PIP_DIR}"

WHEEL_DIR="${WHEEL_DIR:-${SCRIPT_DIR}/../../target/wheels}"
echo "WHEEL_DIR: ${WHEEL_DIR}"

echo "Repairing the wheels"
ls ${WHEEL_DIR}
ls ${WHEEL_DIR}/*.whl | xargs auditwheel show
ls ${WHEEL_DIR}/*.whl | xargs auditwheel repair --wheel-dir ${WHEEL_DIR}/ --plat ${WHEEL_PLATFORM}
rm ${WHEEL_DIR}/*-linux_${WHEEL_ARCH}.whl
ls ${WHEEL_DIR}

echo "Installing the wheels"
ls ${WHEEL_DIR}/*.whl | xargs pip install

pushd ${PIP_DIR}

python -m venv /tmp/.venv

. /tmp/.venv/bin/activate

pip install -r test_requirements.txt

python -m pytest

popd




