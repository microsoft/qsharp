Observe the required state closely, $\ket{\Psi^{3}} = \frac{1}{2}(\ket{000}+\ket{011}+\ket{101}+\ket{110})$. Based on `qAlice` qubit, bell state of `qBob` and `qCharlie` is decided.

- Starting with $\ket{000}$, first step would be to entangle `qBob` and `qCharlie`. This can be done using Hadamard and $CNOT$ operations, similar to `Entangle` exercise. This gives the resultant state as $\frac{1}{\sqrt{2}}(\ket{000} + \ket{011})$
- Applying Hadamard on `qAlice` qubit would give us the state as $\frac{1}{2}(\ket{000} + \ket{011} + \ket{100} + \ket{111})$.
- Finally using $CNOT$, entanglement can be achieved between `qAlice` and `qBob` or `qAlice` and `qCharlie`. This would give the desired result in either of the cases. 

@[solution]({
    "id": "teleportation__entangled_trio_solution",
    "codePath": "./Solution.qs"
})