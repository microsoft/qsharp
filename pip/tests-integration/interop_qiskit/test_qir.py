# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pytest
from qsharp import TargetProfile

from interop_qiskit import QISKIT_AVAILABLE, SKIP_REASON

if QISKIT_AVAILABLE:
    from qsharp.interop import convert_qiskit_to_qir
    from .test_circuits import core_tests, generate_repro_information
else:
    core_tests = []


@pytest.mark.skipif(not QISKIT_AVAILABLE, reason=SKIP_REASON)
@pytest.mark.parametrize("circuit_name", core_tests)
def test_random(circuit_name: str, request):
    circuit = request.getfixturevalue(circuit_name)
    if str.endswith(circuit_name.lower(), "base"):
        target_profile = TargetProfile.Base
    else:
        target_profile = TargetProfile.Adaptive_RI

    try:
        qir = convert_qiskit_to_qir(circuit, target_profile)
        assert qir is not None
    except AssertionError:
        raise
    except Exception as ex:
        additional_info = generate_repro_information(circuit, target_profile)
        raise RuntimeError(additional_info) from ex
