# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os
import json
import numpy as np
from numpy import ndarray


class NoiseModel:
    """
    Represents a noise model used for noisy simulation. This will be written to
    disk as a JSON file with a format similar to the following (replace and repeat
    'n_1q', 'xx', 'Mz', etc. as appropriate)
    {
        "krausOperators": {
            "n_1q": [ ["___", "___"], ["___", "___"] ],
            "n_2q": [ ["___", "___"] ],
        },
        "gates": {
            "xx": ( ["___", "___"], "n_2q" ),
            "zz": ( ["___", "___"], "n_2q" ),
        },
        "instruments": {
            "Mz": [ (["___", "___"], "n_1q", "0"), (["___", "___"], "n_1q", "1") ]
        }
    }

    An "___" is a string representation of a complex array, e.g. "[1.+0.j, 0.+0.j]"
    Quotes are needed as JavaScript/JSON does not support complex numbers natively.

    Noise should be provided WITHOUT the operator included (i.e. noise only models).
    This allows the same noise model (e.g. depolarize by 5%) to be applied to multiple
    operators.
    """

    # Sets of Kraus matrices representing only noise component indexed by noise name
    kraus_operators: dict[str, list[ndarray]]

    # Gates indexed by gate name. Each entry contains gate operator, noise name, and loss probability
    gates: dict[str, tuple[ndarray, str, float]]

    # Measurement gates indexed by gate name. Each entry contains the list of: gate operator, noise name and outcome string
    instruments: dict[str, list[tuple[ndarray, str, str]]]

    # Lists indexed by noise model name. Each List contains gate and instrument names
    noise_models: dict[str, list[str]]

    def __init__(self):
        self.kraus_operators = {}
        self.gates = {}
        self.instruments = {}
        self.rev = 0  # Revision number, used to detect changes in the noise model

    def add_kraus_operator(self, name: str, matrices: list[ndarray]):
        self.kraus_operators[name] = matrices
        self.rev += 1

    def add_gate(
        self, name: str, matrix: ndarray, kraus_name: str, loss_prob: float = 0.0
    ):
        self.gates[name] = (matrix, kraus_name, loss_prob)
        self.rev += 1

    def update_gate_matrix(self, name: str, matrix: ndarray):
        if name in self.gates:
            (old_matrix, kraus_name, loss_prob) = self.gates[name]
            self.gates[name] = (matrix, kraus_name, loss_prob)
            self.rev += 1
        else:
            raise RuntimeError(f"Gate '{name}' not found.")

    def update_gate_noise(self, name: str, kraus_name: str):
        if name in self.gates:
            (matrix, _, loss_prob) = self.gates[name]
            self.gates[name] = (matrix, kraus_name, loss_prob)
            self.rev += 1
        else:
            raise RuntimeError(f"Gate '{name}' not found.")

    def update_gate_loss(self, name: str, loss_prob: float):
        if name in self.gates:
            (matrix, kraus_name, _) = self.gates[name]
            self.gates[name] = (matrix, kraus_name, loss_prob)
            self.rev += 1
        else:
            raise RuntimeError(f"Gate '{name}' not found.")

    def get_noise_matrices_for_gate(self, name: str) -> list[ndarray]:
        """
        Returns the list of Kraus matrices for the specified gate.
        If the gate does not have a noise model, returns an empty list.
        """
        if name in self.gates:
            (_, noise_name, _) = self.gates[name]
            return self.kraus_operators.get(noise_name)
        raise RuntimeError(f"Noise for gate '{name}' not found.")

    def add_instrument(self, name: str, instrument: list[tuple[ndarray, str, str]]):
        self.instruments[name] = instrument
        self.rev += 1

    def add_noise_model(self, gates: list[str], name: str = ""):
        self.noise_models[name] = gates
        self.rev += 1

    def save_config(self, file_path):
        with open(file_path, "w") as file:
            val = {
                "krausOperators": self.kraus_operators,
                "gates": self.gates,
                "instruments": self.instruments,
            }
            json.dump(val, file, indent=2, cls=NumpyMatrixEncoder)

    def load_config(self, config_file_path):
        if not os.path.exists(config_file_path):
            raise FileNotFoundError(f"Config file '{config_file_path}' does not exist")

        with open(config_file_path, "r", encoding="utf-8") as file:
            result = json.load(file, object_hook=load_matrices_from_json)
            if (
                "krausOperators" not in result
                or "gates" not in result
                or "instruments" not in result
            ):
                raise ValueError("Invalid noise model file")

            self.kraus_operators = result["krausOperators"]
            self.gates = result["gates"]
            self.instruments = result["instruments"]
            self.rev += 1

    def get_noisy_gates_and_instruments(self, noise_model_name: str = ""):
        noisy_gates = {}
        noisy_instruments = {}
        for item_name in self.gates:
            gate = self.gates.get(item_name)
            (noiseless_matrix, noise_name, _) = gate
            # NOTE: We assume that both noiseless matrix and noise only matrices are specified in the noise model.
            # We can potentially relax this condition. We don't need to require both.
            # Such relaxation is potentially dangerous - it may hide typos in dictionary keys.
            noise_matrices = self.kraus_operators.get(noise_name)
            if noise_matrices is None:
                raise RuntimeError(
                    f"Kraus operators '{noise_name}' are not defined in the noise model."
                )
            combined_matrices = self.apply_unitary_to_kraus(
                noiseless_matrix, noise_matrices
            )
            noisy_gates[item_name] = combined_matrices
        for item_name in self.instruments:
            instrument = self.instruments.get(item_name)
            choices = []
            for noiseless_matrix, noise_name, output_string in instrument:
                noise_matrices = self.kraus_operators.get(noise_name)
                if noise_matrices is None:
                    raise RuntimeError(
                        f"Kraus operators '{noise_name}' are not defined in the noise model."
                    )
                combined_matrices = self.apply_unitary_to_kraus(
                    noiseless_matrix, noise_matrices
                )
                choices.append((combined_matrices, output_string))
            noisy_instruments[item_name] = choices
        return (noisy_gates, noisy_instruments)

    # Implement the below so can pass instances to lru_cache functions
    def __eq__(self, other):
        # Only compares equal to itself
        return self is other

    def __hash__(self):
        # Objects that compare equal must have the same hash
        return hash(id(self))

    ### Matrix manipulations ###

    @staticmethod
    def apply_unitary_to_kraus(U, kraus_ops):
        # If there is no noise specified, just return the unitary itself as the kraus matrices
        # TODO: extend not to require unitary. Just return kraus_ops in this case.
        if kraus_ops is None or len(kraus_ops) == 0:
            return [U]
        else:
            # Apply unitary transformation to each Kraus operator
            return [E @ U for E in kraus_ops]

    @staticmethod
    def reverse_unitary_from_kraus(
        unitary: np.ndarray, combined_kraus_ops: list[np.ndarray]
    ):
        # Given kraus operators for noise that include the unitary operation, extract just the noise operators
        U_dagger = np.conjugate(unitary.T)
        return [F @ U_dagger for F in combined_kraus_ops]


### Utility classes and method for converting NumPy matrices to and from a JSON format ###


class NumpyMatrixEncoder(json.JSONEncoder):
    """
    Use this class during JSON serialization to convert any NumPy arrays to a JSON representation
    """

    def default(self, o):
        if isinstance(o, np.ndarray):
            return [
                np.array2string(row, separator=",", max_line_width=1000000) for row in o
            ]
        return super().default(o)


def json_array_to_numpy_complex_matrix(json_arr):
    """
    Utility to convert a JSON string array to a NumPy matrix. Each strings should
    represent a row of complex numbers and be of the form "[1.+0.j, 0.+0.j]"
    """

    evalMatrix = [eval(row) for row in json_arr]
    return np.array(evalMatrix, dtype=np.complex128)


def load_matrices_from_json(dct: dict):
    """
    Convert the JSON representations of matrices into NumPy matrices
    """
    if "gates" in dct:
        for gName in dct["gates"]:
            (opMatrix, noise) = dct["gates"][gName]
            npArray = json_array_to_numpy_complex_matrix(opMatrix)
            dct["gates"][gName] = (npArray, noise)

    if "instruments" in dct:
        for iName in dct["instruments"]:
            choices = dct["instruments"][iName]
            upd_choices = []
            for opMatrix, noise, output in choices:
                upd_choices.append(
                    (json_array_to_numpy_complex_matrix(opMatrix), noise, output)
                )
            dct["instruments"][iName] = upd_choices

    if "krausOperators" in dct:
        # The value will be a dictionary, with each key being a kraus operator name,
        # and the value an array of arrays of strings - each an ndarray row in string form
        for krausOp in dct["krausOperators"]:
            matrixList = dct["krausOperators"][krausOp]
            newMatrices = []
            for matrix in matrixList:
                newMatrices.append(json_array_to_numpy_complex_matrix(matrix))

            dct["krausOperators"][krausOp] = newMatrices

    return dct


def amplitude_damping_kraus(gamma):
    """
    Generate Kraus operators for the amplitude damping channel.

    Parameters:
    gamma (float): Damping probability (0 <= gamma <= 1)

    Returns:
    list: Kraus operators [K0, K1] as numpy arrays
    """
    if gamma < 0 or gamma > 1:
        raise ValueError("Damping probability gamma must be between 0 and 1.")

    # Define the Kraus operators
    K0 = np.array([[1, 0], [0, np.sqrt(1 - gamma)]], dtype=np.complex128)

    K1 = np.array([[0, np.sqrt(gamma)], [0, 0]], dtype=np.complex128)

    return [K0, K1]


def bitflip_kraus(p):
    """
    Generate Kraus operators for the bit flip noise channel.

    Parameters:
    p (float): Bit flip probability (0 <= p <= 1)

    Returns:
    list: Kraus operators [K0, K1] as numpy arrays
    """
    if p < 0 or p > 1:
        raise ValueError("Bit flip probability p must be between 0 and 1.")

    # Define the Kraus operators
    # K0: no bit flip occurs (with probability sqrt(1-p))
    K0 = np.sqrt(1 - p) * np.array([[1, 0], [0, 1]], dtype=np.complex128)

    # K1: bit flip occurs (with probability sqrt(p))
    K1 = np.sqrt(p) * np.array([[0, 1], [1, 0]], dtype=np.complex128)

    return [K0, K1]


def amplitude_excitation_kraus(gamma):
    """
    Generate Kraus operators for the amplitude excitation channel.

    This is the opposite of amplitude damping - it models spontaneous excitation
    from |0⟩ to |1⟩ state with probability gamma.

    Parameters:
    gamma (float): Excitation probability (0 <= gamma <= 1)

    Returns:
    list: Kraus operators [K0, K1] as numpy arrays
    """
    if gamma < 0 or gamma > 1:
        raise ValueError("Excitation probability gamma must be between 0 and 1.")

    # Define the Kraus operators
    # K0: no excitation occurs
    K0 = np.array([[np.sqrt(1 - gamma), 0], [0, 1]], dtype=np.complex128)

    # K1: excitation occurs (|0⟩ → |1⟩)
    K1 = np.array([[0, 0], [np.sqrt(gamma), 0]], dtype=np.complex128)

    return [K0, K1]


def dephasing_kraus(gamma):
    """
    Generate Kraus operators for the dephasing (phase damping) channel.

    This noise model causes loss of quantum coherence without changing the
    population of the computational basis states |0⟩ and |1⟩.

    Parameters:
    gamma (float): Dephasing probability (0 <= gamma <= 1)

    Returns:
    list: Kraus operators [K0, K1] as numpy arrays
    """
    if gamma < 0 or gamma > 1:
        raise ValueError("Dephasing probability gamma must be between 0 and 1.")

    # Define the Kraus operators
    # K0: no dephasing occurs
    K0 = np.array([[1, 0], [0, np.sqrt(1 - gamma)]], dtype=np.complex128)

    # K1: dephasing occurs (affects off-diagonal terms)
    K1 = np.array([[0, 0], [0, np.sqrt(gamma)]], dtype=np.complex128)

    return [K0, K1]


def dephasing_2q_target_kraus(gamma):
    """
    Generate Kraus operators for 2-qubit dephasing that affects only the target (second) qubit.

    This is useful for modeling noise in 2-qubit gates like CZ where the target qubit
    experiences dephasing while the control qubit remains unaffected.

    Parameters:
    gamma (float): Dephasing probability for the target qubit (0 <= gamma <= 1)

    Returns:
    list: Kraus operators [K0, K1] as 4x4 numpy arrays
    """
    if gamma < 0 or gamma > 1:
        raise ValueError("Dephasing probability gamma must be between 0 and 1.")

    # Identity matrix for the control qubit (unaffected)
    I = np.eye(2, dtype=np.complex128)

    # Single-qubit dephasing operators for the target qubit
    K0_single = np.array([[1, 0], [0, np.sqrt(1 - gamma)]], dtype=np.complex128)
    K1_single = np.array([[0, 0], [0, np.sqrt(gamma)]], dtype=np.complex128)

    # Create 2-qubit Kraus operators using tensor product
    # K0: I ⊗ K0_single (no dephasing on target)
    K0 = np.kron(I, K0_single)

    # K1: I ⊗ K1_single (dephasing on target)
    K1 = np.kron(I, K1_single)

    return [K0, K1]


def single_to_2q_kraus(
    single_qubit_kraus: list[ndarray], target_qubit: int = 1
) -> list[ndarray]:
    """
    Convert single-qubit Kraus operators to 2-qubit Kraus operators where only one qubit is affected.

    This is a general helper function that can take any single-qubit noise model and extend it
    to a 2-qubit system where only the specified qubit experiences the noise.

    Parameters:
    single_qubit_kraus (list[ndarray]): List of 2x2 Kraus operators for single-qubit noise
    target_qubit (int): Which qubit to apply noise to (0 for first qubit, 1 for second qubit)

    Returns:
    list[ndarray]: List of 4x4 Kraus operators for 2-qubit system

    Example:
    # Apply bit flip noise only to the second qubit
    bitflip_ops = bitflip_kraus(0.1)
    two_qubit_bitflip = single_to_2q_kraus(bitflip_ops, target_qubit=1)

    # Apply amplitude damping only to the first qubit
    damping_ops = amplitude_damping_kraus(0.05)
    two_qubit_damping = single_to_2q_kraus(damping_ops, target_qubit=0)
    """
    if target_qubit not in [0, 1]:
        raise ValueError("target_qubit must be 0 (first qubit) or 1 (second qubit)")

    if not single_qubit_kraus:
        raise ValueError("single_qubit_kraus cannot be empty")

    # Verify all operators are 2x2
    for i, op in enumerate(single_qubit_kraus):
        if op.shape != (2, 2):
            raise ValueError(
                f"Kraus operator {i} has shape {op.shape}, expected (2, 2)"
            )

    # Identity matrix for the unaffected qubit
    I = np.eye(2, dtype=np.complex128)

    # Create 2-qubit Kraus operators using tensor product
    two_qubit_kraus = []

    for kraus_op in single_qubit_kraus:
        if target_qubit == 0:
            # Apply noise to first qubit: K ⊗ I
            two_qubit_op = np.kron(kraus_op, I)
        else:
            # Apply noise to second qubit: I ⊗ K
            two_qubit_op = np.kron(I, kraus_op)

        two_qubit_kraus.append(two_qubit_op)

    return two_qubit_kraus


def rz_for_theta(theta: float) -> ndarray:
    # Top left is e^-i*theta/2, bottom right is e^i*theta/2
    return np.array(
        [[np.exp(-1j * theta / 2), 0], [0, np.exp(1j * theta / 2)]], dtype=np.complex128
    )


def create_default_noise_model() -> NoiseModel:
    noise_model = NoiseModel()

    # Add the default (noiseless) Kraus operators
    noise_model.add_kraus_operator("noise_1q", [np.eye(2, dtype=np.complex128)])
    noise_model.add_kraus_operator("noise_2q", [np.eye(4, dtype=np.complex128)])
    noise_model.add_kraus_operator("noise_3q", [np.eye(8, dtype=np.complex128)])

    # Below is equivalent to 100% amplitude damping noise, i.e. set qubit to 0 state
    noise_model.add_kraus_operator(
        "noise_reset",
        [np.array([[1 + 0j, 0], [0, 0]]), np.array([[0 + 0j, 1], [0, 0]])],
    )

    i_matrix = np.eye(2, dtype=np.complex128)
    x_matrix = np.array([[0 + 0j, 1], [1, 0]])
    y_matrix = np.array([[0 + 0j, -1j], [1j, 0]])
    z_matrix = np.array([[1 + 0j, 0], [0, -1]])

    h_matrix = np.array([[1 + 0j, 1], [1, -1]]) / np.sqrt(2)
    # np.sqrt only works on diagonal matrices
    s_matrix = np.sqrt(z_matrix)
    t_matrix = np.sqrt(s_matrix)

    # sx is the square root of x, i.e. sx * sx = x. Note np.sqrt(x) doesn't work here
    sx_matrix = np.array([[0.5 + 0.5j, 0.5 - 0.5j], [0.5 - 0.5j, 0.5 + 0.5j]])

    s_adj_matrix = np.conjugate(s_matrix.T)
    t_adj_matrix = np.conjugate(t_matrix.T)

    cx_matrix = np.array([[1 + 0j, 0, 0, 0], [0, 1, 0, 0], [0, 0, 0, 1], [0, 0, 1, 0]])
    cz_matrix = np.array([[1 + 0j, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, -1]])

    ccx_matrix = np.eye(8, dtype=np.complex128)
    ccx_matrix[6, 6] = 0
    ccx_matrix[7, 7] = 0
    ccx_matrix[6, 7] = 1
    ccx_matrix[7, 6] = 1

    # Add the default unitary operations
    noise_model.add_gate("i", i_matrix, "noise_1q")
    noise_model.add_gate("move", i_matrix, "noise_1q")
    noise_model.add_gate("x", x_matrix, "noise_1q")
    noise_model.add_gate("y", y_matrix, "noise_1q")
    noise_model.add_gate("z", z_matrix, "noise_1q")
    noise_model.add_gate("h", h_matrix, "noise_1q")
    noise_model.add_gate("s", s_matrix, "noise_1q")
    noise_model.add_gate("t", t_matrix, "noise_1q")
    noise_model.add_gate("s_adj", s_adj_matrix, "noise_1q")
    noise_model.add_gate("t_adj", t_adj_matrix, "noise_1q")
    noise_model.add_gate("sx", sx_matrix, "noise_1q")
    noise_model.add_gate("cx", cx_matrix, "noise_2q")
    noise_model.add_gate("cz", cz_matrix, "noise_2q")
    noise_model.add_gate("ccx", ccx_matrix, "noise_3q")
    # Add the reset operation, which is a gate with 100% amplitude damping noise
    noise_model.add_gate("reset", i_matrix, "noise_reset")

    # NOTE: This about how to generalize gates that take parameters
    noise_model.add_gate(
        "rz", i_matrix, "noise_1q"
    )  # Rz gate is special, needs to be handled separately

    # Add the measurement 'instruments'
    mz_matrix_0 = np.array([[1 + 0j, 0], [0, 0]])
    mz_matrix_1 = np.array([[0 + 0j, 0], [0, 1]])
    noise_model.add_instrument(
        "mz", [(mz_matrix_0, "noise_1q", "0"), (mz_matrix_1, "noise_1q", "1")]
    )

    return noise_model
