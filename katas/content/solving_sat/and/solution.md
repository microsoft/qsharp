The goal is to flip the qubit $\ket{y}$ if and only if all qubits in the register $\ket{x}$ are in the state $\ket{1}$.

The required unitary $U_{and}$ is such that:
$$U_{and}\ket{x}\ket{y} = \begin{cases} 
          \ket{x}\ket{y} & \text{if }x \neq 1...1 \\
          \ket{x}X\ket{y} & \text{if }x = 1...1 
       \end{cases}$$

This transformation can be implemented as a `Controlled X` gate, with the input register $\ket{x}$ as control and the target qubit $\ket{y}$ as target. 

@[solution]({
    "id": "solving_sat__and_solution",
    "codePath": "./Solution.qs"
})
