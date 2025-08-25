# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import json
from typing import Any, Dict, List, Optional, Union

from ...estimator import EstimatorParams, EstimatorResult
from ..._native import OutputSemantics, ProgramType, QasmError
from .backends import QSharpBackend, ResourceEstimatorBackend, QirTarget
from .jobs import QsJob, QsSimJob, ReJob, QsJobSet
from .execution import DetaultExecutor
from qiskit import QuantumCircuit


def estimate(
    circuit: QuantumCircuit,
    params: Optional[Union[Dict[str, Any], List, EstimatorParams]] = None,
    **options,
) -> EstimatorResult:
    from ..._qsharp import ipython_helper

    ipython_helper()
    backend = ResourceEstimatorBackend()
    job = backend.run(circuit, params=params, **options)
    return job.result()
