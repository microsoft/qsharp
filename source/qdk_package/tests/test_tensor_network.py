# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from dataclasses import FrozenInstanceError
import json
import os
from pathlib import Path
import subprocess
import sys
from typing import Any, cast

import pytest

import qdk.simulation
from qdk import _native
from qdk._adaptive_pass import AdaptiveProfilePass, Bytecode
from qdk._native import Result
from qdk.simulation import (
    Cost,
    Expectation,
    MpsCostResult,
    MpsOptions,
    Sample,
    run_qir,
    tensornetwork_qir,
)
from qdk.simulation._simulation import DecomposeCcxPass, preprocess_simulation_input

# Arguments are validated before the program is parsed, so these tests need no
# valid program.
QIR = "not parsed"

BELL_BASE_QIR = """\
%Result = type opaque
%Qubit = type opaque

define void @ENTRYPOINT__main() #0 {
entry:
    call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
    call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
    call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
    call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
    call void @__quantum__rt__array_record_output(i64 2, i8* null)
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
    ret void
}

declare void @__quantum__qis__h__body(%Qubit*)
declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
declare void @__quantum__qis__mz__body(%Qubit*, %Result*)
declare void @__quantum__rt__array_record_output(i64, i8*)
declare void @__quantum__rt__result_record_output(%Result*, i8*)

attributes #0 = { "entry_point" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="2" }
"""


# Ry(2π/5) on qubit 0, then CNOT: cos(π/5)|00⟩ + sin(π/5)|11⟩, so P(11) =
# sin²(π/5) ≈ 0.35. Unlike Bell's, its Schmidt coefficients differ, so χ = 1
# keeps exactly |00⟩.
RY_CX_BASE_QIR = """\
%Result = type opaque
%Qubit = type opaque

define void @ENTRYPOINT__main() #0 {
entry:
    call void @__quantum__qis__ry__body(double 1.2566370614359172, %Qubit* inttoptr (i64 0 to %Qubit*))
    call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
    call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
    call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
    call void @__quantum__rt__array_record_output(i64 2, i8* null)
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
    ret void
}

declare void @__quantum__qis__ry__body(double, %Qubit*)
declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
declare void @__quantum__qis__mz__body(%Qubit*, %Result*)
declare void @__quantum__rt__array_record_output(i64, i8*)
declare void @__quantum__rt__result_record_output(%Result*, i8*)

attributes #0 = { "entry_point" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="2" }
"""


ORDERED_RESULTS_BASE_QIR = """\
%Result = type opaque
%Qubit = type opaque

define void @ENTRYPOINT__main() #0 {
entry:
    call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
    call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
    call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
    call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
    call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
    call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
    ret void
}

declare void @__quantum__qis__x__body(%Qubit*)
declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
declare void @__quantum__qis__mz__body(%Qubit*, %Result*)
declare void @__quantum__rt__tuple_record_output(i64, i8*)
declare void @__quantum__rt__result_record_output(%Result*, i8*)

attributes #0 = { "entry_point" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="2" }
"""


NO_OUTPUT_RECORDING_BASE_QIR = """\
%Result = type opaque
%Qubit = type opaque

define void @ENTRYPOINT__main() #0 {
entry:
    call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
    call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
    ret void
}

declare void @__quantum__qis__x__body(%Qubit*)
declare void @__quantum__qis__mz__body(%Qubit*, %Result*)

attributes #0 = { "entry_point" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="2" }
"""


# One unitary region, then reset and measurement: reaches the consumer's reset.
X_RESET_MEASURE_BASE_QIR = """\
%Result = type opaque
%Qubit = type opaque

define void @ENTRYPOINT__main() #0 {
entry:
    call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
    call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 0 to %Qubit*))
    call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
    ret void
}

declare void @__quantum__qis__x__body(%Qubit*)
declare void @__quantum__qis__reset__body(%Qubit*)
declare void @__quantum__qis__mz__body(%Qubit*, %Result*)
declare void @__quantum__rt__result_record_output(%Result*, i8*)

attributes #0 = { "entry_point" "qir_profiles"="base_profile" "required_num_qubits"="1" "required_num_results"="1" }
"""


# Sx on two qubits, Cz, then measure-and-reset both; qubit 2 is never used.
SX_CZ_MRESETZ_BASE_QIR = """\
%Result = type opaque
%Qubit = type opaque

define void @ENTRYPOINT__main() #0 {
entry:
    call void @__quantum__qis__sx__body(%Qubit* inttoptr (i64 0 to %Qubit*))
    call void @__quantum__qis__sx__body(%Qubit* inttoptr (i64 1 to %Qubit*))
    call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
    call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
    call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
    ret void
}

declare void @__quantum__qis__sx__body(%Qubit*)
declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
declare void @__quantum__qis__mresetz__body(%Qubit*, %Result*)
declare void @__quantum__rt__result_record_output(%Result*, i8*)

attributes #0 = { "entry_point" "qir_profiles"="base_profile" "required_num_qubits"="3" "required_num_results"="2" }
"""


UNSUPPORTED_SHARED_EXECUTION_QIR = """\
%Result = type opaque
%Qubit = type opaque

define void @ENTRYPOINT__main() #0 {
entry:
    {instruction}
    ret void
}

{declaration}

attributes #0 = { "entry_point" "qir_profiles"="base_profile" "required_num_qubits"="1" "required_num_results"="1" }
"""


# Every gate callee the QIR lowering maps, on non-adjacent and reversed
# operands. The tensor-network methods accept all of them (CCX through its
# decomposition), so queries on it reach library discovery.
EVERY_GATE_BASE_QIR = """\
%Result = type opaque
%Qubit = type opaque

define void @ENTRYPOINT__main() #0 {
entry:
    call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
    call void @__quantum__qis__y__body(%Qubit* inttoptr (i64 1 to %Qubit*))
    call void @__quantum__qis__z__body(%Qubit* inttoptr (i64 2 to %Qubit*))
    call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
    call void @__quantum__qis__s__body(%Qubit* inttoptr (i64 1 to %Qubit*))
    call void @__quantum__qis__s__adj(%Qubit* inttoptr (i64 2 to %Qubit*))
    call void @__quantum__qis__t__body(%Qubit* inttoptr (i64 0 to %Qubit*))
    call void @__quantum__qis__t__adj(%Qubit* inttoptr (i64 1 to %Qubit*))
    call void @__quantum__qis__sx__body(%Qubit* inttoptr (i64 2 to %Qubit*))
    call void @__quantum__qis__rx__body(double 0.25, %Qubit* inttoptr (i64 0 to %Qubit*))
    call void @__quantum__qis__ry__body(double 0.25, %Qubit* inttoptr (i64 1 to %Qubit*))
    call void @__quantum__qis__rz__body(double 0.25, %Qubit* inttoptr (i64 2 to %Qubit*))
    call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
    call void @__quantum__qis__cy__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
    call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
    call void @__quantum__qis__swap__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
    call void @__quantum__qis__rxx__body(double 0.25, %Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
    call void @__quantum__qis__ryy__body(double 0.25, %Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 2 to %Qubit*))
    call void @__quantum__qis__rzz__body(double 0.25, %Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
    call void @__quantum__qis__ccx__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
    call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
    call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
    call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 2 to %Result*))
    call void @__quantum__rt__array_record_output(i64 3, i8* null)
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 2 to %Result*), i8* null)
    ret void
}

declare void @__quantum__qis__x__body(%Qubit*)
declare void @__quantum__qis__y__body(%Qubit*)
declare void @__quantum__qis__z__body(%Qubit*)
declare void @__quantum__qis__h__body(%Qubit*)
declare void @__quantum__qis__s__body(%Qubit*)
declare void @__quantum__qis__s__adj(%Qubit*)
declare void @__quantum__qis__t__body(%Qubit*)
declare void @__quantum__qis__t__adj(%Qubit*)
declare void @__quantum__qis__sx__body(%Qubit*)
declare void @__quantum__qis__rx__body(double, %Qubit*)
declare void @__quantum__qis__ry__body(double, %Qubit*)
declare void @__quantum__qis__rz__body(double, %Qubit*)
declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
declare void @__quantum__qis__cy__body(%Qubit*, %Qubit*)
declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)
declare void @__quantum__qis__rxx__body(double, %Qubit*, %Qubit*)
declare void @__quantum__qis__ryy__body(double, %Qubit*, %Qubit*)
declare void @__quantum__qis__rzz__body(double, %Qubit*, %Qubit*)
declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)
declare void @__quantum__qis__mz__body(%Qubit*, %Result*)
declare void @__quantum__rt__array_record_output(i64, i8*)
declare void @__quantum__rt__result_record_output(%Result*, i8*)

attributes #0 = { "entry_point" "qir_profiles"="base_profile" "required_num_qubits"="3" "required_num_results"="3" }
"""


# Rx(π/2) on qubit 0 and H on qubit 1: ⟨Y₀⟩ = -sin(π/2) = -1 and ⟨X₁⟩ = 1.
# Y is the only non-symmetric Pauli, so a transposed Y would read ⟨Y₀⟩ = +1.
RX_H_BASE_QIR = """\
%Result = type opaque
%Qubit = type opaque

define void @ENTRYPOINT__main() #0 {
entry:
    call void @__quantum__qis__rx__body(double 1.5707963267948966, %Qubit* inttoptr (i64 0 to %Qubit*))
    call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
    call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
    call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
    ret void
}

declare void @__quantum__qis__rx__body(double, %Qubit*)
declare void @__quantum__qis__h__body(%Qubit*)
declare void @__quantum__qis__mz__body(%Qubit*, %Result*)
declare void @__quantum__rt__result_record_output(%Result*, i8*)

attributes #0 = { "entry_point" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="2" }
"""


def _adaptive_program(qir: str) -> dict:
    mod, _, _, _ = preprocess_simulation_input(qir, 1, None, 0)
    DecomposeCcxPass().run(mod)
    return AdaptiveProfilePass(Bytecode.Bit64).run(mod).as_dict()


# GPU tests run only with QDK_GPU_TESTS set and a working cuTensorNet backend,
# found by a probe (the MPS method needs at least two qubits). Validation tests
# below run everywhere: they fail before device discovery.
CUTENSORNET_AVAILABLE = False
CUTENSORNET_SKIP_REASON = "QDK_GPU_TESTS not set"
if os.environ.get("QDK_GPU_TESTS"):
    try:
        tensornetwork_qir(BELL_BASE_QIR, [Sample(1)])
        CUTENSORNET_AVAILABLE = True
    except OSError as error:
        CUTENSORNET_SKIP_REASON = str(error)

requires_cutensornet = pytest.mark.skipif(
    not CUTENSORNET_AVAILABLE, reason=CUTENSORNET_SKIP_REASON
)


def test_api_is_publicly_exported():
    for name in (
        "tensornetwork_qir",
        "Sample",
        "Expectation",
        "Cost",
        "MpsCostResult",
        "MpsOptions",
    ):
        assert name in qdk.simulation.__all__
    assert MpsCostResult.__annotations__ == {
        "max_bond_dimension": int,
        "state_bytes": int,
        "workspace_bytes": int,
    }


def test_mps_options_select_the_cutensornet_backend_by_default():
    assert MpsOptions() == MpsOptions(backend=None, max_bond_dimension=None)
    assert MpsOptions(backend="cutensornet").backend == "cutensornet"
    assert MpsOptions(max_bond_dimension=8).max_bond_dimension == 8


@pytest.mark.parametrize("bond", [0, -1, 2**32, True, 2.5, "8"])
def test_mps_options_reject_invalid_bond_dimensions(bond: Any):
    with pytest.raises(ValueError, match="max_bond_dimension"):
        MpsOptions(max_bond_dimension=bond)


def test_mps_options_are_frozen():
    options = MpsOptions(backend="cutensornet", max_bond_dimension=2)
    with pytest.raises(FrozenInstanceError):
        setattr(options, "max_bond_dimension", 4)


def test_sample_keeps_its_shot_count():
    assert Sample(8).shots == 8


@pytest.mark.parametrize("shots", [0, -1, 2**32, True, False, 2.5, "8", None])
def test_sample_rejects_invalid_shots(shots: Any):
    with pytest.raises(ValueError, match="shots"):
        Sample(shots)


def test_expectation_normalizes_terms():
    query = Expectation([("ZZ", [0, 1], 0.5), ("X", (2,), 1)])
    assert query.terms == (("ZZ", (0, 1), 0.5 + 0j), ("X", (2,), 1 + 0j))


@pytest.mark.parametrize(
    "term",
    [
        ("ZA", [0, 1], 1.0),
        ("", [], 1.0),
        ("ZZ", [0], 1.0),
        ("ZZ", [1, 1], 1.0),
        ("Z", [-1], 1.0),
        ("Z", [True], 1.0),
    ],
)
def test_expectation_rejects_invalid_terms(term: Any):
    with pytest.raises(ValueError):
        Expectation([term])


@pytest.mark.parametrize("term", [("Z", [0]), ("Z", [0], "1"), ("Z", [0], True)])
def test_expectation_rejects_malformed_terms(term: Any):
    with pytest.raises(TypeError):
        Expectation([term])


def test_expectation_requires_a_term():
    with pytest.raises(ValueError, match="at least one term"):
        Expectation([])


def test_rejects_unknown_method():
    with pytest.raises(ValueError, match="Invalid method"):
        tensornetwork_qir(QIR, [Cost()], method=cast(Any, "contraction"))


def test_rejects_options_of_another_type():
    with pytest.raises(TypeError, match="MpsOptions"):
        tensornetwork_qir(QIR, [Cost()], options=cast(Any, {}))


@pytest.mark.parametrize("backend", ["cpu", "amd", "nvidia"])
def test_rejects_unsupported_backends(backend: str):
    options = MpsOptions(backend=cast(Any, backend))
    with pytest.raises(ValueError, match="Unsupported MPS backend"):
        tensornetwork_qir(QIR, [Sample(1)], options=options)


@pytest.mark.parametrize("seed", [-1, 2**32, True, 2.5, "42"])
def test_rejects_invalid_seeds(seed: Any):
    with pytest.raises(ValueError, match="seed"):
        tensornetwork_qir(QIR, [Sample(1)], seed=seed)


def test_requires_a_query():
    with pytest.raises(ValueError, match="at least one query"):
        tensornetwork_qir(QIR, [])


def test_rejects_unknown_queries():
    with pytest.raises(TypeError, match="Unsupported query"):
        tensornetwork_qir(QIR, [cast(Any, "cost")])


def test_rejects_more_than_one_sample():
    with pytest.raises(ValueError, match="at most one Sample"):
        tensornetwork_qir(QIR, [Sample(1), Sample(2)])


@pytest.mark.parametrize(
    "queries",
    [
        [Sample(1), Cost()],
        [Cost(), Expectation([("Z", [0], 1)]), Sample(1)],
    ],
)
def test_sample_with_cost_is_planned_for_the_next_release(queries: list):
    with pytest.raises(ValueError, match="next release"):
        tensornetwork_qir(QIR, queries)


@pytest.mark.parametrize("profile", ["adaptive_profile", "custom_profile"])
def test_sample_of_a_non_base_profile_is_planned_for_the_next_release(profile: str):
    qir = BELL_BASE_QIR.replace(
        '"qir_profiles"="base_profile"', f'"qir_profiles"="{profile}"'
    )
    with pytest.raises(ValueError, match=f"found '{profile}'.*next release"):
        tensornetwork_qir(qir, [Sample(1)])


def test_sample_rejects_reset_before_discovery():
    with pytest.raises(
        ValueError, match="reset is not supported by cuTensorNet MPS batch sampling"
    ):
        tensornetwork_qir(X_RESET_MEASURE_BASE_QIR, [Sample(1)], seed=42)


@pytest.mark.parametrize(
    "opcode,instruction,declaration",
    [
        (
            "16",
            "call void @__quantum__qis__peek_loss__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))",
            "declare void @__quantum__qis__peek_loss__body(%Qubit*, %Result*)",
        ),
        (
            "17",
            "call void @__quantum__rt__readout_noise(double 0.0, double 0.0, %Result* inttoptr (i64 0 to %Result*))",
            "declare void @__quantum__rt__readout_noise(double, double, %Result*)",
        ),
    ],
    ids=["OP_PEEK_LOSS_0x16", "OP_READOUT_NOISE_0x17"],
)
def test_sample_rejects_unsupported_opcodes_before_discovery(
    opcode: str, instruction: str, declaration: str
):
    qir = UNSUPPORTED_SHARED_EXECUTION_QIR.replace(
        "{instruction}", instruction
    ).replace("{declaration}", declaration)
    with pytest.raises(
        ValueError, match=rf"unsupported adaptive opcode 0x{opcode} at instruction 0"
    ):
        tensornetwork_qir(qir, [Sample(1)])


def test_state_query_rejects_invalid_native_terms_before_discovery():
    with pytest.raises(ValueError, match="unknown Pauli label"):
        _native._tensor_network_state_query(
            _adaptive_program(BELL_BASE_QIR),
            [{"kind": "expectation", "terms": [("ZQ", [0, 1], 1 + 0j)]}],
        )


def test_expectation_rejects_an_absent_qubit_before_discovery():
    with pytest.raises(ValueError, match="qubit 5, but the program has 2 qubits"):
        tensornetwork_qir(BELL_BASE_QIR, [Expectation([("Z", [5], 1)])])


def test_expectation_rejects_reset_before_discovery():
    with pytest.raises(ValueError, match="reset"):
        tensornetwork_qir(X_RESET_MEASURE_BASE_QIR, [Expectation([("Z", [0], 1)])])


def test_queries_report_unavailable_libraries_as_oserror(tmp_path: Path):
    # Process construction injects the supported discovery override without
    # mutating this process's environment or requiring any particular GPU host.
    # A program with every gate shows that none is rejected, and one whose
    # measurements reset shows that terminal measure-and-reset is accepted.
    script = """
import json
import platform
import sys
from qdk.simulation import Cost, Expectation, Sample, tensornetwork_qir
programs = json.loads(sys.stdin.read())
bell, gates, mresetz = programs["bell"], programs["gates"], programs["mresetz"]
zz = Expectation([("ZZ", [0, 1], 1)])
cases = (
    (bell, [Sample(1)]),
    (bell, [zz]),
    (bell, [Cost()]),
    (bell, [zz, Sample(1)]),
    (gates, [Sample(1)]),
    (gates, [zz, Cost()]),
    (mresetz, [Sample(1)]),
    (mresetz, [zz, Cost()]),
)
for qir, queries in cases:
    try:
        tensornetwork_qir(qir, queries, seed=42)
    except OSError as error:
        expected = ("QDK_CUTENSORNET_LIBRARY"
                    if sys.platform == "linux" and platform.machine() == "x86_64"
                    else "unsupported on")
        assert expected in str(error), str(error)
    else:
        raise AssertionError("missing library must not be a successful query")
"""
    result = subprocess.run(
        [sys.executable, "-c", script],
        input=json.dumps(
            {
                "bell": BELL_BASE_QIR,
                "gates": EVERY_GATE_BASE_QIR,
                "mresetz": SX_CZ_MRESETZ_BASE_QIR,
            }
        ),
        text=True,
        capture_output=True,
        env={
            **os.environ,
            "QDK_CUTENSORNET_LIBRARY": str(tmp_path / "not-installed.so"),
        },
    )
    assert result.returncode == 0, result.stderr


@requires_cutensornet
def test_sample_preserves_ordered_results_across_shots():
    shots = 8
    expected = run_qir(ORDERED_RESULTS_BASE_QIR, shots=shots, seed=42, type="cpu")
    (actual,) = tensornetwork_qir(ORDERED_RESULTS_BASE_QIR, [Sample(shots)], seed=42)
    assert actual == expected == [(Result.One, Result.Zero)] * shots


@requires_cutensornet
def test_sample_reports_all_results_without_output_recording():
    expected = run_qir(NO_OUTPUT_RECORDING_BASE_QIR, shots=1, seed=42, type="cpu")
    (actual,) = tensornetwork_qir(NO_OUTPUT_RECORDING_BASE_QIR, [Sample(1)], seed=42)
    assert actual == expected == ["10"]


@requires_cutensornet
def test_seeded_bell_samples_are_reproducible_and_correlated():
    (first,) = tensornetwork_qir(BELL_BASE_QIR, [Sample(100)], seed=42)
    (second,) = tensornetwork_qir(BELL_BASE_QIR, [Sample(100)], seed=42)
    assert first == second
    assert len({tuple(outcome) for outcome in first}) > 1
    assert all(
        outcome in ([Result.Zero, Result.Zero], [Result.One, Result.One])
        for outcome in first
    )


@requires_cutensornet
def test_sample_honors_the_bond_dimension():
    (uncapped,) = tensornetwork_qir(RY_CX_BASE_QIR, [Sample(100)], seed=42)
    assert [Result.One, Result.One] in uncapped
    (capped,) = tensornetwork_qir(
        RY_CX_BASE_QIR,
        [Sample(100)],
        options=MpsOptions(max_bond_dimension=1),
        seed=42,
    )
    assert capped == [[Result.Zero, Result.Zero]] * 100


@requires_cutensornet
def test_sample_and_expectation_preserve_query_order():
    zz = Expectation([("ZZ", [0, 1], 1)])
    first, shots, last = tensornetwork_qir(
        BELL_BASE_QIR, [zz, Sample(10), zz], seed=42
    )
    assert first == pytest.approx(1)
    assert last == pytest.approx(1)
    (expected,) = tensornetwork_qir(BELL_BASE_QIR, [Sample(10)], seed=42)
    assert shots == expected


@requires_cutensornet
def test_expectations_match_bell_and_single_qubit_states():
    zz, xx, yy, z0, weighted = tensornetwork_qir(
        BELL_BASE_QIR,
        [
            Expectation([("ZZ", [0, 1], 1)]),
            Expectation([("XX", [0, 1], 1)]),
            Expectation([("YY", [0, 1], 1)]),
            Expectation([("Z", [0], 1)]),
            Expectation([("II", [0, 1], 0.5), ("ZZ", [0, 1], 2j)]),
        ],
    )
    assert zz == pytest.approx(1)
    assert xx == pytest.approx(1)
    assert yy == pytest.approx(-1)
    assert z0 == pytest.approx(0, abs=1e-12)
    assert weighted == pytest.approx(0.5 + 2j)
    y0, x1 = tensornetwork_qir(
        RX_H_BASE_QIR, [Expectation([("Y", [0], 1)]), Expectation([("X", [1], 1)])]
    )
    assert y0 == pytest.approx(-1)
    assert x1 == pytest.approx(1)


@requires_cutensornet
def test_cost_reports_the_realized_bond_and_preserves_query_order():
    cost, zz, cost_again = tensornetwork_qir(
        BELL_BASE_QIR,
        [Cost(), Expectation([("ZZ", [0, 1], 1)]), Cost()],
        options=MpsOptions(max_bond_dimension=4),
    )
    assert zz == pytest.approx(1)
    assert cost == cost_again
    assert set(cost) == {"max_bond_dimension", "state_bytes", "workspace_bytes"}
    # Two sites of extents [2, 2] at 16 bytes per complex128 element.
    assert cost["max_bond_dimension"] == 2
    assert cost["state_bytes"] == 128
    assert cost["workspace_bytes"] > 0
    (capped,) = tensornetwork_qir(
        BELL_BASE_QIR, [Cost()], options=MpsOptions(max_bond_dimension=1)
    )
    assert capped["max_bond_dimension"] == 1
