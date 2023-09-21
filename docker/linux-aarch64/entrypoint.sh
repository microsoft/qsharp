#!/usr/bin/env bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

set -e

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

echo "Setting up the virtual environment"
python3 -m venv /tmp/.venv
. /tmp/.venv/bin/activate


echo "Update pip"
pip install -U pip

echo "Installing auditwheel and patchelf"
pip install auditwheel patchelf

echo "Repairing the wheels"
ls ${WHEEL_DIR}
ls ${WHEEL_DIR}/*.whl | xargs auditwheel show
ls ${WHEEL_DIR}/*.whl | xargs auditwheel repair --wheel-dir ${WHEEL_DIR}/ --plat ${WHEEL_PLATFORM}
rm ${WHEEL_DIR}/*-linux_${WHEEL_ARCH}.whl
ls ${WHEEL_DIR}

echo "Installing the wheels"
ls ${WHEEL_DIR}/*.whl | xargs pip install

pushd ${PIP_DIR}

pip install -r test_requirements.txt

pushd tests

python3 -m pytest

popd

popd
