This task is more interesting: the given gates differ by a global phase they introduce (i.e., each of them is a multiple of the other), and the results of applying them to any single-qubit state are going to be indistinguishable by any measurement you can devise.

Fortunately, we are given not just the unitary itself, but also its controlled variant, i.e., the gate which applies the given unitary if the control qubit or qubits are in the $\ket{1}$ state, and does nothing if they are in the $\ket{0}$ state.
This allows us to use so called "phase kickback" trick, in which applying a controlled version of a gate allows us to observe the phase introduced by this gate on the control qubit. Indeed,

| State | Controlled Z | Controlled $-Z$ |
|-------|---------------|------|
| $\ket{00}$ | $\ket{00}$ | $\ket{00}$ |
| $\ket{01}$ | $\ket{01}$ | $\ket{01}$ |
| $\ket{10}$ | $\color{blue}{\ket{10}}$  | $\color{blue}{-\ket{10}}$ |
| $\ket{11}$ | $\color{blue}{-\ket{11}}$ | $\color{blue}{\ket{11}}$  |

We see that both controlled gates don't modify the states with the control qubit in the $\ket{0}$ state, but if the control qubit is in the $\ket{1}$ state, they introduce a $-1$ phase to different basis states. 
We can take advantage of this if we apply the controlled gate to a state in which the *control qubit* is in superposition, such as $\frac{1}{\sqrt2}(\ket{0} + \ket{1}) \otimes \ket{0}$:

$$\text{Controlled Z}\frac{1}{\sqrt2}(\ket{0} + \ket{1}) \otimes \ket{0} = \frac{1}{\sqrt2}(\ket{0} + \ket{1}) \otimes \ket{0}$$
$$\text{Controlled }-\text{Z}\frac{1}{\sqrt2}(\ket{0} + \ket{1}) \otimes \ket{0} = \frac{1}{\sqrt2}(\ket{0} - \ket{1}) \otimes \ket{0}$$

After this we can measure the first qubit to distinguish $\frac{1}{\sqrt2}(\ket{0} + \ket{1})$ from $\frac{1}{\sqrt2}(\ket{0} - \ket{1})$, like we did in task 'Identity or Pauli-Z'.

> In Q# we can express controlled version of a gate using [Controlled functor](https://learn.microsoft.com/en-us/azure/quantum/user-guide/language/expressions/functorapplication#controlled-functor): the first argument of the resulting gate will be an array of control qubits, and the second one - the arguments of the original gate (in this case just the target qubit).

@[solution]({
    "id": "distinguishing_unitaries__z_mz_solution",
    "codePath": "Solution.qs"
})
