Recall that this state is the Bell state that we have already seen in Multi-Qubit Gates and Superposition katas.

The solution takes two steps:

1. Apply Hadamard gate on first qubit to get the state: $\big(\frac{1}{\sqrt{2}}|0\rangle + \frac{1}{\sqrt{2}}|1\rangle\big) \otimes |0\rangle$.
2. Apply a $CNOT$ gate with the first qubit as the control and the second qubit as the target.

@[solution]({
    "id": "superdense_coding__create_entangled_pair_solution",
    "codePath": "./Solution.qs"
})
