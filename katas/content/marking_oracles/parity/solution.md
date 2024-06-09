We can write $f(x)$ as follows:

$$f(x) = x_0 \oplus x_1 \oplus ... \oplus x_{N-1}$$

Let's substitute this expression in the expression for the oracle effect on the quantum state:

$$U_f \ket{x} \ket{y} = \ket{x} \ket{y \oplus f(x)} = \ket{x} \ket{y \oplus x_0 \oplus x_1 \oplus ... \oplus x_{N-1}}$$

Now, we can represent the final state of the target qubit as a result of a series of $N$ marking oracles applied sequentially, marking oracle $j$ flipping its state if the state of the input qubit $j$ $\ket{x_j}$ is $\ket{1}$.
As we saw in the previous task, each of these marking oracles can be implemented as a single $CNOT$ gate.

@[solution]({
    "id": "marking_oracles__parity_solution",
    "codePath": "./Solution.qs"
})
