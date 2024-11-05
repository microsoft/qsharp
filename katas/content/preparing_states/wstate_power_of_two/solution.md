The problem becomes more manageable if broken down into the simplest cases and built up from there.

1. The smallest instance of the problem, $N = 1$, requires preparing $\ket{W_1} = \ket{1}$; this can be done trivially using an $X$ gate.

2. The next instance, $N = 2$, requires preparing $\ket{W_2} = \frac{1}{\sqrt2}\big(\ket{10} + \ket{01}\big)$. It matches one of the Bell states you've seen earlier, but preparing it will be more interesting (and more useful for the next steps!) if you think of it in recursive terms. Let's see how to express $\ket{W_2}$ in terms of $\ket{W_1}$:

$$\ket{W_2} = \frac{1}{\sqrt2}\big(\ket{10} + \ket{01}\big) = \frac{1}{\sqrt2}\big(\ket{W_1} \otimes \ket{0} + \ket{0} \otimes \ket{W_1}\big)$$

This representation suggests you a solution: "split" the starting state $\ket{00}$ in two terms, prepare $\ket{W_1}$ on the first qubit for the first term and on the second qubit - for the second term.
To do this, you can again use an auxiliary qubit prepared in the $\ket{+}$ state and control the preparation of $\ket{W_1}$ state on the first or the second qubit based on the state of the auxiliary qubit:

$$\ket{0}_{aux} \ket{00}_{reg} \overset{H}{\longrightarrow} \frac{1}{\sqrt2}(\ket{0}_{aux} + \ket{1}_{aux}) \otimes \ket{00}_{reg} = \frac{1}{\sqrt2}(\ket{0}_{aux} \ket{00}_{reg} + \ket{1}_{aux} \ket{00}_{reg})\overset{CNOT_0}{\longrightarrow}$$

$${\longrightarrow}\frac{1}{\sqrt2}(\ket{0}_{aux} \ket{W_1}\ket{0}_{reg} + \ket{1}_{aux} \ket{00}_{reg})\overset{CNOT_1}{\longrightarrow}$$

$${\longrightarrow}\frac{1}{\sqrt2}(\ket{0}_{aux} \ket{W_1}\ket{0}_{reg} + \ket{1}_{aux} \ket{0}\ket{W_1}_{reg})$$

> The auxiliary qubit is now entangled with the rest of the qubits, so you can't simply reset it without it affecting the superposition you have prepared using it.

The last step to bring the register to the desired state is to uncompute the auxiliary qubit for the term $\ket{1}_{aux} \ket{0}\ket{W_1}_{reg}$ (the other term already has it in state $\ket{0}$).

To do this, you need to consider the explicit expression of the state $\ket{0}\ket{W_1} = \ket{01}$. Similarly to the previous tasks, you'll uncompute the auxiliary qubit for this term by using a controlled $X$ gate, with the auxiliary qubit as the target and the main register in the $\ket{01}$ state as a control. This will make sure that the gate is applied only for this term and not for any others.

The last step can be simplified to use fewer qubits as controls: you can use just the second qubit of the main register in state $\ket{1}$ as control, since you know that if the second qubit is in state $\ket{1}$, the first one has to be in state $\ket{0}$ (you don't need to use both of them as the control pattern).

3. If you take this one step further, to $N = 4$, you'll see that the same recursive logic can be applied to the larger and larger sizes of the problem. Indeed,

$$\ket{W_4} = \frac{1}{2}\big(\ket{1000} + \ket{0100} + \ket{0010} + \ket{0001}\big) = $$
$$= \frac{1}{\sqrt2} \big(\frac{1}{\sqrt2}(\ket{10} + \ket{01}) \otimes \ket{00} + \ket{00} \otimes \frac{1}{\sqrt2}(\ket{10} + \ket{01}) \big) = $$
$$= \frac{1}{\sqrt2} \big(\ket{W_2} \otimes \ket{00} + \ket{00} \otimes \ket{W_2}\big)$$

You can use the same approach for this case: prepare an auxiliary qubit in $\ket{+}$ state and use it to control preparation of $W_2$ state on the first and the second half of the register. The last step will be uncomputing the $\ket{1}$ state of the auxiliary qubit using two controlled $X$ gates with each of the qubits of the second half of the register in state $\ket{1}$ as controls.

The same recursive approach can be generalized for arbitrary powers of 2 as the register size.

@[solution]({
    "id": "preparing_states__wstate_power_of_two_solution_a",
    "codePath": "./SolutionA.qs"
})

This implementation of the recursion requires $\log_2 N$ extra qubits allocated for controlling the preparation (one per level of recursion). You can modify your approach to use just one extra qubit at a time.

To do this, let's notice that to prepare $\ket{W_{N}}$ you need to prepare the $\ket{W_{N-1}}$ state on half of the qubits for both states of the auxiliary qubit, the difference is just in which half of the register you're using. This means that you can prepare the $\ket{W_{N-1}}$ state on the first half of the qubits, and use an auxiliary qubit in superposition to control swapping the first half of the register with the second half. The uncomputation of the auxiliary qubit happens in the same way as in the first approach.

@[solution]({
    "id": "preparing_states__wstate_power_of_two_solution_b",
    "codePath": "./SolutionB.qs"
})
