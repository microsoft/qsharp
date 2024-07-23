# Copyright (c) Microsoft Corporation. All rights reserved.
# Licensed under the MIT License.

import pytest
import qsharp
from qsharp.utils import dump_operation

@pytest.fixture(autouse=True)
def setup():
    """Fixture to execute asserts before a test is run"""
    # You can set the target profile here if you want
    # By default, it uses "Unrestricted" profile
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    yield # this is where the testing happens

def test_empty_operation() -> None:
    res = dump_operation("qs => ()", 1)
    assert res == [
        [complex(1.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(1.0, 0.0)],
    ]


def test_single_qubit_not_gate() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)

    res = dump_operation("qs => X(qs[0])", 1)
    assert res == [
        [complex(0.0, 0.0), complex(1.0, 0.0)],
        [complex(1.0, 0.0), complex(0.0, 0.0)],
    ]

def test_single_qubit_hadamard_gate() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)

    res = dump_operation("qs => H(qs[0])", 1)
    assert res == [
        [complex(0.707107, 0.0), complex(0.707107, 0.0)],
        [complex(0.707107, 0.0), complex(-0.707107, 0.0)],
    ]

def test_two_qubit_cnot_gate() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)

    res = dump_operation("qs => CNOT(qs[0], qs[1])", 2)
    assert res == [
        [complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0)],
    ]

def test_custom_operation() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)

    qsharp.eval(
        "operation ApplySWAP(qs : Qubit[]) : Unit is Ctl + Adj { SWAP(qs[0], qs[1]); }"
    )

    res = dump_operation("ApplySWAP", 2)
    assert res == [
        [complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0)],
    ]

def test_operation_no_args_in_qsharp_file() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    qsharp.init(project_root='.')

    res = dump_operation("qs => CustomOperation.ApplySWAP(qs[0], qs[1])", 2)
    assert res == [
        [complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(1.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.0, 0.0), complex(0.0, 0.0), complex(1.0, 0.0)],
    ]

def test_operation_with_args_in_qsharp_file() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    qsharp.init(project_root='.')

    res0 = dump_operation("BellState.AllBellStates(_, 0)", 2)

    assert res0 == [
        [complex(0.707107, 0.0), complex(0.0,0.0), complex(0.707107, 0.0), complex(0.0,0.0)],
        [complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0)],
        [complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0), complex(-0.707107, 0.0)],
        [complex(0.707107, 0.0), complex(0.0, 0.0), complex(-0.707107, 0.0), complex(0.0, 0.0)],
    ]

    res1 = dump_operation("BellState.AllBellStates(_, 1)", 2)

    assert res1 == [
        [complex(0.707107, 0.0), complex(0.0,0.0), complex(0.707107, 0.0), complex(0.0,0.0)],
        [complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0)],
        [complex(0.0, 0.0), complex(-0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0)],
        [complex(-0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0)],
    ]

    res2 = dump_operation("BellState.AllBellStates(_, 2)", 2)

    assert res2 == [
        [complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0)],
        [complex(0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0)],
        [complex(0.707107, 0.0), complex(0.0, 0.0), complex(-0.707107, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0), complex(-0.707107, 0.0)],
    ]

    res3 = dump_operation("BellState.AllBellStates(_, 3)", 2)

    assert res3 == [
        [complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0)],
        [complex(0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0)],
        [complex(-0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0), complex(0.0, 0.0)],
        [complex(0.0, 0.0), complex(-0.707107, 0.0), complex(0.0, 0.0), complex(0.707107, 0.0)],
    ]


def test_operation_equivalence_using_fact() -> None:
    qsharp.init(target_profile=qsharp.TargetProfile.Unrestricted)
    qsharp.init(project_root='.')

    qsharp.eval(
        "OperationEquivalence.TestEquivalence()"
    )
