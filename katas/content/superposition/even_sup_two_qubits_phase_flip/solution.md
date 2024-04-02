Here we start with the end state of the previous task $\frac{1}{2} \big(|00\rangle + |01\rangle + |10\rangle + |11\rangle\big)$. Looking at the desired state, the phase of the $|11\rangle$ state is flipped ($+$ changed to a $-$).

A regular phase flip on one qubit can be done using a Z gate:
$$\begin{bmatrix} 1 & 0 \\\ 0 & -1 \end{bmatrix}$$
This gate will perform a phase flip only on the $|1\rangle$ state:

$$Z(\alpha|0\rangle + \beta|1\rangle) = \alpha|0\rangle - \beta|1\rangle$$

In our case we only want to flip the phase of the $|11\rangle$ state and not the $|01\rangle$ state. To accomplish this, we can use a controlled Z gate; this will make sure that the $Z$ gate is only applied if the control bit is in the $|1\rangle$ state, and the $|01\rangle$ state will not change.

> In Q# we can apply a controlled gate by using the `Controlled` keyword before the gate. The controlled gate will take two parameters; the first parameter is an array of control qubits (you can have multiple qubits used as a control), and the second parameter is a tuple of parameters passed to the original gate (in this case it's just the qubit to which you want to apply the gate if the control bit is $|1\rangle$).

@[solution]({
    "id": "superposition__even_sup_two_qubits_phase_flip_solution",
    "codePath": "./Solution.qs"
})
