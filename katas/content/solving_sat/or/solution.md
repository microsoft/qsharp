The goal is to flip the qubit $\ket{y}$ if and only if at least one of the qubits in the register $\ket{x}$ is in the state $\ket{1}$.

The required unitary $U_{or}$ is such that:
$$U_{or}\ket{x}\ket{y} = \begin{cases} 
          \ket{x}\ket{y} & \text{if }x = 0...0 \\
          \ket{x}X\ket{y} & \text{if }x \neq 0...0
       \end{cases}$$

This transformation can be implemented as a sequence of two steps:

1. Flip the state of the target qubit if $x = 0...0$ using a controlled-on-zero $X$ gate.
2. Flip the state of the target qubit using an $X$ gate. This will negate the results of the previous step,
   making sure that overall the state of the target qubit is flipped if $x \neq 0...0$.

@[solution]({
    "id": "solving_sat__or_solution",
    "codePath": "./Solution.qs"
})
