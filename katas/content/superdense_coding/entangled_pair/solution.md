Recall that this state is the Bell state that we have already seen in "Multi-Qubit Gates" and "Preparing Quantum States" katas.

The solution takes two steps:

1. Apply Hadamard gate on first qubit to get the state: $\big(\frac{1}{\sqrt{2}} \ket{0} + \frac{1}{\sqrt{2}} \ket{1} \big) \otimes \ket{0}$.
2. Apply a $CNOT$ gate with the first qubit as the control and the second qubit as the target.

@[solution]({
    "id": "superdense_coding__create_entangled_pair_solution",
    "codePath": "./Solution.qs"
})
