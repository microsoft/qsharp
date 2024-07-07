To identify the qubit on which the error happened, you need to do two parity measurements in the $ZZ$ basis on any two different pairs of qubits and analyze their outcomes: 

- If both parity measurements yield $0$, no error occurred.
- If both parity measurements yield $1$, the error occurred on the qubit that is shared between the measured pairs of qubits.
- If one of the parity measurements yields $1$ and the other $0$, the error occurred on the qubit that was part of only the pair of qubits involved in the measurement that yields $1$ but not the other pair.

The code below implements this logic using $ZZ$ parity measurements on pairs of qubits $0, 1$ and $1, 2$.

@[solution]({
    "id": "qec_shor__bitflip_detect_solution",
    "codePath": "Solution.qs"
})
