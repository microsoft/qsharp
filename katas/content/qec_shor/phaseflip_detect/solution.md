To identify the qubit on which the error happened, we can use the same logic as we did for the error detection for the bit flip code. We need to do two parity measurements, this time in the $X$ basis, on any two different pairs of qubits and analyze their outcomes.
The code below implements this logic using $XX$ parity measurements on pairs of qubits $0, 1$ and $1, 2$.

@[solution]({
    "id": "qec_shor__phaseflip_detect_solution",
    "codePath": "Solution.qs"
})
