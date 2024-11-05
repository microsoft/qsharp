> A common strategy for preparing a superposition state in a qubit register is using an auxiliary qubit (or several, for more complicated states). The auxiliary qubit can be put into a superposition state through the usual means of applying a Hadamard gate (or a rotation about the $Y$ axis for an uneven superposition). 
> Then the basis states of the desired superposition are prepared individually based on the auxiliary qubit state by using it as the control qubit for a $CNOT$ gate. One of the basis states will be prepared controlled on the $\ket{0}$ component of the auxiliary state, and the other - controlled on the $\ket{1}$ component. 
> Finally, you have to return the auxiliary qubit to the $\ket{0}$ state by uncomputing it, that is, by using the basis state prepared from the $\ket{1}$ component as the control qubits for a $CNOT$ gate with the auxiliary qubit as the target. 
>
> More details on using this approach can be found in the solution to "Superposition of Four Bitstrings" and "W State on $2^k$ Qubits" tasks. However, for this task you can come up with a simpler solution. 
> Instead of allocating a new qubit to use as the auxiliary, you can use the first qubit in the register for this purpose, because you're guaranteed that the first bit in the two basis vectors that comprise the required superposition is different.
> This saves you the need to allocate a new qubit and lets us skip the uncomputing step, as the qubit acting as the control for the next preparation steps is part of the desired result.

Consider the earlier tasks in this kata that asked to prepare Bell states and GHZ state; the structure of the superposition state in this task is a more general case of those scenarios: all of them ask to prepare an equal superposition of two different basis states.

The first step of the solution is the same as in those tasks: put the first qubit in the register into an equal superposition of $\ket{0}$ and $\ket{1}$ using the $H$ gate to get the following state:

$$\frac{1}{\sqrt2} (\ket{0} + \ket{1}) \otimes \ket{0 \dots 0} = \frac{1}{\sqrt2} (\ket{00 \dots 0} + \ket{10 \dots 0})$$

The first term of the superposition already matches the desired state, so you need to fix the second term.
To do that, you'll walk through the remaining qubits in the register, checking if the bit in the corresponding position of the bit string `bits` is `true`. 
If it is, that qubit's state needs to be adjusted from $0$ to $1$ in the second term of the superposition (and left unchanged in the first term). 
You can do this change using the $CNOT$ gate with the first qubit as the control and the current qubit as the target.
When you have finished walking through the register like this, the register will be in the desired superposition.

@[solution]({
    "id": "preparing_states__zero_and_bitstring_solution",
    "codePath": "./Solution.qs"
})
