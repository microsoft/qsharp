For a basis state $\ket{x_0 ... x_{N-1}}$ to be a palindrome, the state of the first qubit $x_0$ has to be the same as that of the last qubit $x_{N-1}$, the state of the second qubit $x_1$ - the same as that of the second-to-last qubit $x_{N-2}$, and so on.
In other words, we need to compute XORs of pairs of qubits, and if all of these XORs are $0$, the basis state is a palindrome.

Recall that to compute XOR of the states of two qubits in-place, you can use a $CNOT$ gate with one of them as control and the other as target. After this, the state of the target qubit will be exactly the XOR:

$$CNOT \ket{a}\ket{b} = \ket{a}\ket{a \oplus b}$$

We can use this to calculate the XORs of $x_{N-1}$ and $x_0$ (stored in qubit `x[0]`), $x_{N-2}$ and $x_1$ (stored in qubit `x[1]`), and so on. Finally, to check that all XORs are $0$, we can use the library operation `ApplyControlledOnInt` to apply a controlled-on-zero $X$ gate with the qubits we used to calculate XORs as controls and the target qubit as the target.

Remember to uncompute the changes you did to the qubits of the input register. You can do this easily using the within-apply pattern that takes care of uncomputation automatically.

@[solution]({
    "id": "marking_oracles__palindrome_solution",
    "codePath": "./Solution.qs"
})
