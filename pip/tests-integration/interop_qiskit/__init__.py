# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

try:
    # pylint: disable=unused-import
    import qiskit

    QISKIT_AVAILABLE = True
except ImportError:
    QISKIT_AVAILABLE = False

SKIP_REASON = "Qiskit is not available"
