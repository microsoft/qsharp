# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from textwrap import dedent
from qsharp.noisy_simulator import (
    NoisySimulatorError,
    Operation,
    Instrument,
    DensityMatrixSimulator,
    StateVectorSimulator,
)
import pytest


# Tests for the Q# noisy simulator.


def test_operation_api():
    op = Operation([[[1, 0], [0, 0]]])
    assert op.get_number_of_qubits() == 1
    assert op.get_kraus_operators() == [[(1 + 0j), 0j, 0j, 0j]]
    assert op.get_effect_matrix() == [(1 + 0j), 0j, 0j, 0j]
    assert op.get_operation_matrix() == [
        (1 + 0j),
        0j,
        0j,
        0j,
        0j,
        0j,
        0j,
        0j,
        0j,
        0j,
        0j,
        0j,
        0j,
        0j,
        0j,
        0j,
    ]


def test_instrument_api():
    mz0 = Operation([[[1, 0], [0, 0]]])
    mz1 = Operation([[[0, 0], [0, 1]]])
    mz = Instrument([mz0, mz1])


def test_density_matrix_simulator():
    f = 0.5**0.5
    h = Operation([[[f, f], [f, -f]]])
    mz0 = Operation([[[1, 0], [0, 0]]])
    mz1 = Operation([[[0, 0], [0, 1]]])
    mz = Instrument([mz0, mz1])

    sim = DensityMatrixSimulator(1, seed=42)
    sim.apply_operation(h, [0])

    # Applying MZ twice should yield the same result
    outcome_0 = sim.sample_instrument(mz, [0])
    outcome_1 = sim.sample_instrument(mz, [0])
    assert outcome_0 == outcome_1


def test_density_matrix_simulator_out_of_bounds_qubit():
    f = 0.5**0.5
    h = Operation([[[f, f], [f, -f]]])
    sim = DensityMatrixSimulator(1)

    with pytest.raises(NoisySimulatorError) as excinfo:
        sim.apply_operation(h, [1])

    assert str(excinfo.value) == "qubit id out of bounds: 1"


def test_state_vector_simulator():
    f = 0.5**0.5
    h = Operation([[[f, f], [f, -f]]])
    mz0 = Operation([[[1, 0], [0, 0]]])
    mz1 = Operation([[[0, 0], [0, 1]]])
    mz = Instrument([mz0, mz1])

    sim = StateVectorSimulator(1, seed=42)
    sim.apply_operation(h, [0])

    # Applying MZ twice should yield the same result
    outcome_0 = sim.sample_instrument(mz, [0])
    outcome_1 = sim.sample_instrument(mz, [0])
    assert outcome_0 == outcome_1


def test_state_vector_simulator_out_of_bounds_qubit():
    f = 0.5**0.5
    h = Operation([[[f, f], [f, -f]]])
    sim = StateVectorSimulator(1)

    with pytest.raises(NoisySimulatorError) as excinfo:
        sim.apply_operation(h, [1])

    assert str(excinfo.value) == "qubit id out of bounds: 1"
