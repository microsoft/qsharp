From the goal of the exercise, you can see that the register $j$ has to remain unchanged, while the first input qubit should acquire a phase that depends on the value of the qubits in the register $j$.

Since $j$ is a quantum register and can be in a superposition of basis states, you cannot just measure the register and then apply the operation from the previous task using measurement results as the second argument. Instead, you have to convert the solution to the previous task from using $\textrm{R1Frac}$ gates with classical conditions to using them as controlled operations, with the qubits of the register $j$ as quantum conditions. You can do this using the `Controlled` functor.

@[solution]({
"id": "qft__binary_fraction_quantum_solution",
"codePath": "./Solution.qs"
})
