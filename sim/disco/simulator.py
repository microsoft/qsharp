from qsharp.noisy_simulator import StateVectorSimulator, Operation, Instrument
from .noise_model import create_default_noise_model, amplitude_damping_kraus


class QirSim:
    def __init__(self, qubit_count: int = 10, seed: int = 0):
        self.qubit_count = qubit_count
        self.seed = seed
        self.noise_model = create_default_noise_model()

        # Put some amplitude damping noise on the H gate
        self.noise_model.add_kraus_operator("amp_damp_h", amplitude_damping_kraus(0.02))
        (old_h_matrix, _) = self.noise_model.gates["h"]
        self.noise_model.gates["h"] = (old_h_matrix, "amp_damp_h")

        self.simulator = StateVectorSimulator(qubit_count, seed)
        self.results = {}
        self.trace = []  # TODO: Actually trace execution

        (gates, instruments) = self.noise_model.get_noisy_gates_and_instruments()

        self.operations: dict[str, Operation] = {}
        self.measurements: dict[str, Instrument] = {}
        self.outcomes: dict[str, list[str]] = {}

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

    def apply_operation(self, op_name: str, qubits: list[int]):
        qubits.reverse()
        noisy_gate = self.operations.get(op_name)
        if noisy_gate is not None:
            self.simulator.apply_operation(noisy_gate, qubits)
        else:
            raise "Unsupported operation"

    def apply_instrument(self, op_name: str, qubits: list[int], resultReg: list[int]):
        qubits.reverse()
        resultReg.reverse()
        # TODO: Handle multiple result registers (e.g. gadget to measure 2 qubits in X basis)
        instrument = self.measurements.get(op_name)
        outcomes = self.outcomes.get(op_name)
        if instrument is not None:
            outcome_idx = self.simulator.sample_instrument(instrument, qubits)
            outcome = outcomes[outcome_idx]
            if outcome is None:
                raise f"No outcome for index {outcome_idx} returned by instrument {op_name}"
            self.results[resultReg[0]] = outcome
        else:
            raise "Unsupported instrument"

    def reset_simulator(self):
        self.seed += 1
        self.simulator = StateVectorSimulator(self.qubit_count, self.seed)
        self.results = {}
        self.trace = []
