# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""⑤ Python API: ``tensornetwork_qir`` evaluates queries on a QIR program with
the MPS method, one value per query.

::

    tensornetwork_qir(qir, [Sample(n), Expectation(terms), Cost()], method="mps")
      │ host: check arguments and queries; lower QIR to 64-bit bytecode
      ├─ Sample ───────────► _tensor_network_sample       ─► ④ run_mps_shots
      │                        └─ each shot's records ─► output recording (as run_qir)
      └─ Expectation, Cost ► _tensor_network_state_query  ─► ④ evaluate_state_queries
                                                        ─► one value per query, in order

- This module checks the arguments and the queries, prepares the program as
  the other QIR simulators do (``DecomposeCcxPass``, then
  ``AdaptiveProfilePass(Bytecode.Bit64)``), and formats shot results as
  ``run_qir`` does. The simulation is delegated to ④, the MPS method on the
  cuTensorNet backend, through the bindings in
  ``src/qir_simulation/tensor_network_state.rs``. ④ checks the program on the
  host, before device discovery, and reports environment failures (no
  library, no device, a resource limit) as ``OSError`` and unsupported
  operations or invalid input as ``ValueError``.
- ``Sample`` uses the batch strategy: one MPS, every shot sampled from it.
  ``Expectation`` reads |ψ⟩, the state before the terminal measurements;
  ``Cost`` reports the resources used by the call. ``Sample`` with
  ``Expectation`` makes two independent native calls, so each evolves its own
  MPS.

Temporary limits, whose errors say that support comes in the next release:

- ``Sample`` accepts only Base-profile QIR (``_validate_base_profile``). M1's
  ``mps-a`` adds Adaptive programs, with the incremental strategy, and removes
  the guard.
- ``Cost`` cannot be combined with ``Sample``. M1's ④ ``finish`` reports the
  bond reached and the discarded weight in ``Cost`` for ``Sample`` and
  ``Expectation``, and removes the error.

Preview: names, arguments, and result formats may change.
"""

from dataclasses import dataclass
from numbers import Number
from typing import Any, Iterable, List, Literal, Optional, Sequence, Tuple, TypedDict, Union

import pyqir
from pyqir import Module

from .._adaptive_pass import AdaptiveProfilePass, Bytecode
from .._types import QirInputData
from ._simulation import (
    DecomposeCcxPass,
    OutputRecordingPass,
    preprocess_simulation_input,
)

PauliTerm = Tuple[str, Tuple[int, ...], complex]

_U32_MAX = 2**32 - 1


class MpsCostResult(TypedDict):
    """Resources used to construct and query one MPS."""

    max_bond_dimension: int
    state_bytes: int
    workspace_bytes: int


@dataclass(frozen=True)
class MpsOptions:
    """Options for the MPS method of :func:`tensornetwork_qir`.

    Preview: these options may change.

    :param backend: ``"cutensornet"`` or ``None``; both select the cuTensorNet
        backend, on an NVIDIA GPU. Other backends, such as one on the CPU,
        come later.
    :param max_bond_dimension: Largest bond dimension χ the MPS may keep.
        Smaller values are faster and use less memory but truncate more, so
        results are approximate. ``None`` keeps the backend default of 128.
    """

    backend: Optional[Literal["cutensornet"]] = None
    max_bond_dimension: Optional[int] = None

    def __post_init__(self) -> None:
        bond = self.max_bond_dimension
        if bond is not None and (
            isinstance(bond, bool)
            or not isinstance(bond, int)
            or not 1 <= bond <= _U32_MAX
        ):
            raise ValueError(
                "max_bond_dimension must be a positive integer below 2**32 or None"
            )


@dataclass(frozen=True)
class Sample:
    """Query ``shots`` shot results, in the format ``run_qir`` returns.

    The MPS method uses the batch strategy: it evolves one MPS and samples
    every shot from it. Only Base-profile programs are accepted for now;
    Adaptive programs are planned for the next release. The ``seed`` of
    :func:`tensornetwork_qir` reproduces the shots.

    :param shots: The number of shots, a positive integer.
    """

    shots: int

    def __post_init__(self) -> None:
        shots = self.shots
        if (
            isinstance(shots, bool)
            or not isinstance(shots, int)
            or not 1 <= shots <= _U32_MAX
        ):
            raise ValueError("shots must be a positive integer below 2**32")


@dataclass(frozen=True, init=False)
class Expectation:
    """Query the expectation value ⟨ψ|O|ψ⟩ of O = Σₖ cₖ·Pₖ.

    |ψ⟩ is the state reached before the program's terminal measurements; a
    mid-circuit measurement is an unsupported operation. Each term is
    ``(paulis, qubits, coefficient)``, in the same order as Qiskit's
    ``SparsePauliOp.from_sparse_list``: ``("ZZ", [0, 1], 0.5)`` is 0.5·Z₀Z₁.
    Qubits are QIR qubit indices. The result is a complex number, real up to
    rounding when O is Hermitian.
    """

    terms: Tuple[PauliTerm, ...]

    def __init__(self, terms: Iterable[Tuple[str, Sequence[int], complex]]) -> None:
        normalized = tuple(_pauli_term(term) for term in terms)
        if not normalized:
            raise ValueError("Expectation requires at least one term")
        object.__setattr__(self, "terms", normalized)


@dataclass(frozen=True)
class Cost:
    """Query the resources of the call's MPS.

    The result is an :class:`MpsCostResult`: ``"max_bond_dimension"`` (the
    largest bond reached), ``"state_bytes"`` and ``"workspace_bytes"``. It
    describes the MPS for |ψ⟩ that :class:`Expectation` reads. Cost cannot be
    combined with :class:`Sample` yet; support is planned for the next release.
    """


Query = Union[Sample, Expectation, Cost]
QueryValue = Union[List[Any], complex, MpsCostResult]


def tensornetwork_qir(
    input: Union[QirInputData, str, bytes],
    queries: Sequence[Query],
    *,
    method: Literal["mps"] = "mps",
    options: Optional[MpsOptions] = None,
    seed: Optional[int] = None,
) -> List[QueryValue]:
    """Evaluate queries on a QIR program with a tensor-network method.

    Preview: this API may change.

    Returns one value per query, in order: shot results for :class:`Sample`,
    a complex number for :class:`Expectation`, and an :class:`MpsCostResult`
    for :class:`Cost`. A call accepts at most one :class:`Sample`, which may be
    combined with :class:`Expectation`; each is then computed from its own MPS.
    :class:`Cost` cannot be combined with :class:`Sample` yet.

    Errors are raised before any GPU work when the arguments or the program
    are unsupported (``ValueError``, or ``TypeError`` for an argument of the
    wrong type), and as ``OSError`` when the backend is unavailable (no
    library or GPU) or runs out of resources.

    :param input: The QIR program. :class:`Sample` requires the Base profile
        for now; Adaptive programs are planned for the next release.
    :param queries: The queries to evaluate: :class:`Sample`,
        :class:`Expectation` and :class:`Cost`.
    :param method: ``"mps"``, the MPS method: it represents the state as a
        matrix product state whose bond dimension is capped by
        ``options.max_bond_dimension``, so results are approximate when the
        cap truncates.
    :param options: :class:`MpsOptions`; ``None`` uses the defaults.
    :param seed: Reproduces the shots of :class:`Sample`; a value in
        [0, 2**32). ``None`` draws a random seed.
    :return: One value per query.
    """
    if method != "mps":
        raise ValueError(f'Invalid method: {method!r}. Use "mps".')
    if options is not None and not isinstance(options, MpsOptions):
        raise TypeError('options must be an MpsOptions instance for method="mps"')
    options = options if options is not None else MpsOptions()
    if options.backend not in (None, "cutensornet"):
        raise ValueError(
            f"Unsupported MPS backend: {options.backend!r}. "
            'Only backend="cutensornet" is accepted.'
        )
    if seed is not None and (
        isinstance(seed, bool) or not isinstance(seed, int) or not 0 <= seed <= _U32_MAX
    ):
        raise ValueError("seed must be an integer in [0, 2**32) or None")

    queries = list(queries)
    if not queries:
        raise ValueError("queries must contain at least one query")
    for query in queries:
        if not isinstance(query, (Sample, Expectation, Cost)):
            raise TypeError(
                f"Unsupported query {query!r}; use Sample, Expectation or Cost"
            )
    samples = [query for query in queries if isinstance(query, Sample)]
    if len(samples) > 1:
        raise ValueError("queries must contain at most one Sample()")
    if samples and any(isinstance(query, Cost) for query in queries):
        raise ValueError(
            "Cost() cannot be combined with Sample() yet; "
            "support is planned for the next release."
        )

    from .. import _native

    mod, _, _, seed = preprocess_simulation_input(input, None, None, seed)
    if samples:
        _validate_base_profile(mod)
    DecomposeCcxPass().run(mod)
    program = AdaptiveProfilePass(Bytecode.Bit64).run(mod).as_dict()
    max_bond_dimension = options.max_bond_dimension

    results: dict[int, QueryValue] = {}
    state_positions = [
        position
        for position, query in enumerate(queries)
        if not isinstance(query, Sample)
    ]
    if state_positions:
        values = _native._tensor_network_state_query(
            program,
            [_encode_query(queries[position]) for position in state_positions],
            max_bond_dimension,
        )
        for position, value in zip(state_positions, values, strict=True):
            query = queries[position]
            assert isinstance(query, (Expectation, Cost))
            results[position] = _decode_state_query_result(query, value)
    if samples:
        records = _native._tensor_network_sample(
            program, samples[0].shots, seed, max_bond_dimension
        )
        recorder = OutputRecordingPass()
        recorder.run(mod)
        position = next(
            position
            for position, query in enumerate(queries)
            if isinstance(query, Sample)
        )
        results[position] = [recorder.process_output(shot) for shot in records]
    return [results[position] for position in range(len(queries))]


def _qir_profile(mod: Module) -> Optional[str]:
    entry = next(filter(pyqir.is_entry_point, mod.functions), None)
    if entry is None:
        return None
    func_attrs = entry.attributes.func
    if "qir_profiles" not in func_attrs:
        return None
    return func_attrs["qir_profiles"].string_value


def _validate_base_profile(mod: Module) -> None:
    errors = mod.verify()
    if errors is not None:
        raise ValueError(f"Module verification failed: {errors}")

    profile = _qir_profile(mod)
    if profile != "base_profile":
        actual = profile if profile is not None else "unspecified"
        raise ValueError(
            f"Sample requires Base-profile QIR for now; found {actual!r}. "
            "Adaptive programs are planned for the next release."
        )


def _pauli_term(term: Any) -> PauliTerm:
    try:
        paulis, qubits, coefficient = term
    except (TypeError, ValueError):
        raise TypeError(
            f"Expectation term {term!r} must be (paulis, qubits, coefficient)"
        ) from None
    if not isinstance(paulis, str) or not paulis or set(paulis) - set("IXYZ"):
        raise ValueError(
            f"Expectation term {term!r}: paulis must be a non-empty string of I, X, Y, Z"
        )
    qubits = tuple(qubits)
    if any(isinstance(q, bool) or not isinstance(q, int) or q < 0 for q in qubits):
        raise ValueError(
            f"Expectation term {term!r}: qubits must be non-negative integers"
        )
    if len(qubits) != len(paulis):
        raise ValueError(f"Expectation term {term!r}: needs one qubit per Pauli")
    if len(set(qubits)) != len(qubits):
        raise ValueError(f"Expectation term {term!r}: qubits must be distinct")
    if isinstance(coefficient, bool) or not isinstance(coefficient, Number):
        raise TypeError(f"Expectation term {term!r}: coefficient must be a number")
    return (paulis, qubits, complex(coefficient))  # type: ignore[arg-type]


def _encode_query(query: Query) -> dict:
    if isinstance(query, Expectation):
        return {
            "kind": "expectation",
            "terms": [
                (paulis, list(qubits), coefficient)
                for paulis, qubits, coefficient in query.terms
            ],
        }
    return {"kind": "cost"}


def _decode_state_query_result(
    query: Union[Expectation, Cost], value: object
) -> Union[complex, MpsCostResult]:
    if isinstance(query, Expectation):
        if not isinstance(value, complex):
            raise RuntimeError("the native backend returned an invalid Expectation result")
        return value

    if not isinstance(value, dict):
        raise RuntimeError("the native backend returned an invalid Cost result")

    fields = ("max_bond_dimension", "state_bytes", "workspace_bytes")
    if any(type(value.get(field)) is not int for field in fields):
        raise RuntimeError("the native backend returned an invalid Cost result")

    return MpsCostResult(
        max_bond_dimension=value["max_bond_dimension"],
        state_bytes=value["state_bytes"],
        workspace_bytes=value["workspace_bytes"],
    )
