There are multiple ways to approach this problem. The first solution uses the recursive approach that was hinted at in the task.

Let's denote the required state on $N$ qubits as $\ket{P_{N,0}}$ for `parity = 0` and $\ket{P_{N,1}}$ for `parity = 1`. You can group the $2^{N-1}$ basis states included in the state $\ket{P_{N,0}}$ by their first bit ($0$ or $1$) and write the state as follows:

$$\ket{P_{N,0}} = \frac{1}{\sqrt{2^{N-1}}} \sum_{k : k \text{ has parity }0} \ket{k}_N = $$
$$= \frac{1}{\sqrt{2^{N-1}}} \big( \ket{0} \otimes \sum_{k' : k' \text{ has parity }0} \ket{k'}_{N-1} + \ket{1} \otimes \sum_{k'' : k'' \text{ has parity }1} \ket{k''}_{N-1} \big) = $$
$$= \frac{1}{\sqrt{2}} \big( \ket{0} \otimes \ket{P_{N-1,0}} + \ket{1} \otimes \ket{P_{N-1,1}} \big)$$
You can consider the expression for $\ket{P_{N,1}}$ in a similar manner, and get a unified expression for both states:
$$\ket{P_{N,p}} = \frac{1}{\sqrt{2}} \big( \ket{0} \otimes \ket{P_{N-1,p}} + \ket{1} \otimes \ket{P_{N-1,1-p}} \big)$$

Now you can use this expression to prepare the state using a recursive approach you've seen before:

1. Apply $H$ gate to the first qubit to prepare state $\frac{1}{\sqrt{2}} ( \ket{0} + \ket{1} ) \otimes \ket{0}_{N-1}$.
2. Apply the controlled variant of procedure of preparing $\ket{P_{N-1,p}}$ on the last $N-1$ qubits, with the first qubit in the $\ket{0}$ state as the control.
3. Apply the controlled variant of procedure of preparing $\ket{P_{N-1,1-p}}$ on the last $N-1$ qubits, with the first qubit in the $\ket{1}$ state as the control.

> Q# library function [`ApplyControlledOnInt`](https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.canon/applycontrolledonint) allows you to do that easily.

4. The base of recursion is preparing the states for $N = 1$:
* For `parity = 0`, there is one single-qubit state with this parity: $\ket{0}$ (no action required to prepare).
* For `parity = 1`, there is one single-qubit state with this parity: $\ket{1}$ (apply $X$ gate to prepare).

@[solution]({ "id": "preparing_states__parity_bitstrings_solution_a", "codePath": "./SolutionA.qs" })

The second solution uses post-selection. You start by preparing an equal superposition of all basis states and allocating an extra qubit.

This time you use the extra qubit to calculate the parity of the input state: applying a series of $CNOT$ gates, each one with one of the input qubits as control and the extra qubit as a target will compute the parity of the state.

Now, you measure the extra qubit: if the measurement result matches the parity, you're done — the input qubits collapsed to an equal superposition of all states that have this parity. If the measurement result is the opposite, you can retry the whole process.

You can avoid retrying the state preparation if your measurement result doesn't match the required parity: notice that applying an $X$ gate to any one of the qubits changes the parity of each basis state to the opposite one, and thus converts the state you got to the state you need.

@[solution]({ "id": "preparing_states__parity_bitstrings_solution_b", "codePath": "./SolutionB.qs" })

Yet another way of getting the desired superposition could be preparing the mix of all possible basis states for all qubits iteratively, keeping the parity on each step.

You start by preparing a superposition of all basis states with parity $0$. To achieve that, you loop through all the qubits except the first one and prepare all of them in equal superposition. In order to maintain the parity of the basis states involved, use the first qubit and conditionally flip its state using a $CNOT$ gate with each next qubit as the control, so that every time there is a $\ket{1}$ state in the chain, you get back to an even number of $\ket{1}$ states in that basis state.

For example, after the first loop iteration, you get the state $\frac12(\ket{00} + \ket{11})$. After the second iteration, you get the state

$$CNOT_{2,0} \frac1{\sqrt2}(\ket{00} + \ket{11}) \otimes \frac1{\sqrt2}(\ket{0} + \ket{1}) = $$
$$= CNOT_{2,0} \frac12(\ket{000} + \ket{\textbf{0}0\textbf{1}} + \ket{110} + \ket{\textbf{1}1\textbf{1}}) = $$
$$= \frac12(\ket{000} + \ket{101} + \ket{110} + \ket{011})$$

After the loop you'll have a superposition of all possible basis states with even number of $\ket{1}$s.
Then, if `parity` is equal to 1 and we want an odd number of $\ket{1}$s, you just flip the state of the first qubit again (or any qubit).

For example, if the input has 3 qubits, after the for loop you'll have a superposition of 4 basis states:

$$\frac12(\ket{000} + \ket{101} + \ket{110} + \ket{011})$$

If `parity = 0`, you're done, having even numbers of $\ket{1}$s. If `parity = 1`, you flip the state of the first qubit, getting the desired result:

$$\frac12(\ket{100} + \ket{001} + \ket{010} + \ket{111})$$

@[solution]({ "id": "preparing_states__parity_bitstrings_solution_c", "codePath": "./SolutionC.qs" })
