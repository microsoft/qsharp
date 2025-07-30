from functools import lru_cache
from qsharp import Result
from qsharp.noisy_simulator import StateVectorSimulator, Instrument, Operation
from qsharp._qsharp import QirInputData
from .noise_model import (
    NoiseModel,
    create_default_noise_model,
    amplitude_damping_kraus,
    rz_for_theta,
)
from .ops import QirOps


# This function is used to get the RZ operator for a given angle.
# It uses lru_cache to cache the results for performance.
# rev_list is used for cache busting when the noise model is updated.
@lru_cache(maxsize=64)
def get_rz_operator(model: NoiseModel, rev: int, theta: float) -> Operation:
    op_matrix = rz_for_theta(theta)
    noise_matrices = model.get_noise_matrices_for_gate("rz")
    combined_matrices = model.apply_unitary_to_kraus(op_matrix, noise_matrices)
    return Operation(combined_matrices)


def to_results(shots: list[str]) -> list[list[Result]]:
    """
    Convert a list of shot results (strings) to a list of lists of Result.
    Each character in the string corresponds to a Result (0 or 1).
    """
    result_list = []
    for shot in shots:
        result = [Result.Zero if bit == "0" else Result.One for bit in shot]
        result_list.append(result)
    return result_list


class Simulator:
    def __init__(self, noise_model: NoiseModel = create_default_noise_model()):
        self.noise_model = noise_model

    def run(self, code: QirInputData | str, shots: int = 1):
        seed = 0
        if isinstance(code, QirInputData):
            code = str(code)

        ops = QirOps(code)
        self.shots = []

        for shot in range(shots):
            seed += 1
            self.simulator = StateVectorSimulator(ops.qubits, seed)

            # Do this here rather than init, so the noise model can be modified between runs.
            (gates, instruments) = self.noise_model.get_noisy_gates_and_instruments()

            self.operations: dict[str, Operation] = {}
            self.measurements: dict[str, Instrument] = {}
            self.outcomes: dict[str, list[str]] = {}
            self.results = {}
            self.reported_results = []

            for gate_name in gates.keys():
                self.operations[gate_name.lower()] = Operation(gates[gate_name])

            for measurement_name in instruments.keys():
                projectors = []
                results_str: list[str] = []
                for choice in instruments[measurement_name]:
                    projectors.append(Operation(choice[0]))
                    results_str.append(choice[1])
                self.measurements[measurement_name.lower()] = Instrument(projectors)
                self.outcomes[measurement_name.lower()] = results_str

            for op in ops.ops:
                if op.name == "initialize" or op.name == "barrier":
                    pass
                elif op.name == "rz":
                    angle = op.args[0]
                    sim_op = get_rz_operator(
                        self.noise_model, self.noise_model.rev, angle
                    )
                    self.simulator.apply_operation(sim_op, [op.args[1]])
                elif op.name in self.operations:
                    self.apply_operation(op.name, op.args)
                elif op.name in self.measurements:
                    # We only support a single qubit and result for now
                    self.apply_instrument(op.name, [op.args[0]], [op.args[1]])
                elif op.name == "array_record_output":
                    pass
                elif op.name == "result_record_output":
                    self.reported_results.append(self.results.get(op.args[0]))
                else:
                    raise ValueError(f"Unsupported operation {op.name} in QIR code")

            result_list = [self.results.get(i) for i in range(ops.results)]
            self.shots.append("".join(result_list))
        return self.shots

    def apply_operation(self, op_name: str, qubits: list[int]):
        noisy_gate = self.operations.get(op_name)
        if noisy_gate is None:
            raise f"Unsupported operation {op_name}"

        self.simulator.apply_operation(noisy_gate, qubits[::-1])

    def apply_instrument(self, op_name: str, qubits: list[int], resultReg: list[int]):
        # TODO: Handle multiple result registers (e.g. gadget to measure 2 qubits in X basis)
        instrument = self.measurements.get(op_name)
        outcomes = self.outcomes.get(op_name)
        if instrument is not None:
            outcome_idx = self.simulator.sample_instrument(instrument, qubits[::-1])
            outcome = outcomes[outcome_idx]
            if outcome is None:
                raise f"No outcome for index {outcome_idx} returned by instrument {op_name}"
            self.results[resultReg[0]] = outcome
        else:
            raise "Unsupported instrument"
