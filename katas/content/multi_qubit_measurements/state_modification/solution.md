Note that if you measure the first qubit in the computational basis, then an outcome of $0$ collapses the second qubit to the state $a\ket 0 + b \ket 1$, while an outcome of $1$ collapses the second qubit to the state $b\ket 0 + a \ket 1$.

Thus, if $ind=0$ and you measure $0$ or if $ind=1$ and we measure $1$, then after the measurement the second qubit will be in the desired state. On the other hand, if $ind=1$ and you measure $0$, or if $ind=0$ and you measure $1$, then the state of the second qubit after the measurement isn't what you're looking for, but you can adjust it using the Pauli X gate.

@[solution]({
    "id": "multi_qubit_measurements__state_modification_solution",
    "codePath": "Solution.qs"
})
