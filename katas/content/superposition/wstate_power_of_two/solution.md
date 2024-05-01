The problem becomes more manageable if broken down into the simplest cases and built up from there.

1. The smallest instance of the problem, $N = 1$, requires preparing $|W_1\rangle = |1\rangle$; this can be done trivially using an X gate.

2. The next instance, $N = 2$, requires preparing $|W_2\rangle = \frac{1}{\sqrt2}\big(|10\rangle + |01\rangle\big)$. It matches one of the Bell states we've seen earlier, but preparing it will be more interesting (and more useful for the next steps!) if we think of it in recursive terms. Let's see how to express $|W_2\rangle$ in terms of $|W_1\rangle$:

$$|W_2\rangle = \frac{1}{\sqrt2}\big(|10\rangle + |01\rangle\big) = \frac{1}{\sqrt2}\big(|W_1\rangle \otimes |0\rangle + |0\rangle \otimes |W_1\rangle\big)$$

This representation suggests us a solution: "split" the starting state $|00\rangle$ in two terms, prepare $|W_1\rangle$ on the first qubit for the first term and on the second qubit - for the second term.
To do this, we can again use an auxiliary qubit prepared in the $|+\rangle$ state and control the preparation of $|W_1\rangle$ state on the first or the second qubit based on the state of the auxiliary qubit:

$$|0\rangle_{aux} |00\rangle_{reg} \overset{H}{\longrightarrow}
\frac{1}{\sqrt2}(|0\rangle + |1\rangle)_{aux} \otimes |00\rangle_{reg} =
\frac{1}{\sqrt2}(|0\rangle_{aux} |00\rangle_{reg} + |1\rangle_{aux} |00\rangle_{reg})
\overset{CNOT_0}{\longrightarrow} $$
$${\longrightarrow}\frac{1}{\sqrt2}(|0\rangle_{aux} |W_1\rangle|0\rangle_{reg} + |1\rangle_{aux} |00\rangle_{reg})
\overset{CNOT_1}{\longrightarrow} $$
$${\longrightarrow}\frac{1}{\sqrt2}(|0\rangle_{aux} |W_1\rangle|0\rangle_{reg} + |1\rangle_{aux} |0\rangle|W_1\rangle_{reg})$$

> The auxiliary qubit is now entangled with the rest of the qubits, so we can't simply reset it without it affecting the superposition we have prepared using it.

The last step to bring the register to the desired state is to uncompute the auxiliary qubit for the term $|1\rangle_{aux} |0\rangle|W_1\rangle_{reg}$ (the other term already has it in state $|0\rangle$).

To do this, we need to consider the explicit expression of the state $|0\rangle|W_1\rangle = |01\rangle$. Similarly to the previous tasks, we'll uncompute the auxiliary qubit for this term by using a controlled X gate, with the auxiliary qubit as the target and the main register in the $|01\rangle$ state as a control. This will make sure that the gate is applied only for this term and not for any others.

The last step can be simplified to use fewer qubits as controls: we can use just the second qubit of the main register in state $|1\rangle$ as control, since we know that if the second qubit is in state $|1\rangle$, the first one has to be in state $|0\rangle$ (we don't need to use both of them as the control pattern).

3. If we take this one step further, to $N = 4$, we'll see that the same recursive logic can be applied to the larger and larger sizes of the problem. Indeed,

$$|W_4\rangle = \frac{1}{2}\big(|1000\rangle + |0100\rangle + |0010\rangle + |0001\rangle\big) = \\\\
= \frac{1}{\sqrt2} \big(\frac{1}{\sqrt2}(|10\rangle + |01\rangle) \otimes |00\rangle + |00\rangle \otimes \frac{1}{\sqrt2}(|10\rangle + |01\rangle) \big) = \\\\
= \frac{1}{\sqrt2} \big(|W_2\rangle \otimes |00\rangle + |00\rangle \otimes |W_2\rangle\big)
$$

We can use the same approach for this case: prepare an auxiliary qubit in $|+\rangle$ state and use it to control preparation of $W_2$ state on the first and the second half of the register. The last step will be uncomputing the $|1\rangle$ state of the auxiliary qubit using two controlled X gates with each of the qubits of the second half of the register in state $|1\rangle$ as controls.

The same recursive approach can be generalized for arbitrary powers of 2 as the register size.

@[solution]({
    "id": "superposition__wstate_power_of_two_solution_a",
    "codePath": "./SolutionA.qs"
})

This implementation of the recursion requires $\log_2 N = k$ extra qubits allocated for controlling the preparation (one per level of recursion). We can modify our approach to use just one extra qubit at a time.

To do this, let's notice that to prepare $|W_{N}\rangle$ we need to prepare the $|W_{N-1}\rangle$ state on half of the qubits for both states of the auxiliary qubit, the difference is just in which half of the register we're using. This means that we can prepare the $|W_{N-1}\rangle$ state on the first half of the qubits, and use an auxiliary qubit in superposition to control SWAP-ing the first half of the register with the second half. The uncomputation of the auxiliary qubit happens in the same way as in the first approach.

    TODO code 2
