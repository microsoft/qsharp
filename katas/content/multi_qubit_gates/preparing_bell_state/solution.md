We've seen this state before in the[Multi-Qubit Systems kata, where we established that this state is not separable, i.e., it can not be prepared using just the single-qubit gates. To prepare it, we need to use `CNOT` gate.

Let's look at the effect of the `CNOT` gate on a separable state described in the tutorial:
$$CNOT_{1,2}\big(\alpha|0\rangle + \beta|1\rangle\big) \otimes |0\rangle = CNOT_{1,2}(\alpha|00\rangle + \beta|10\rangle) = \alpha|00\rangle + \beta|11\rangle$$

This resulting state is exactly the state we need to prepare, with $\alpha = \beta = \frac{1}{\sqrt{2}}$!

The solution takes two steps:
1. Prepare a state $\big(\frac{1}{\sqrt{2}}|0\rangle + \frac{1}{\sqrt{2}}|1\rangle\big) \otimes |0\rangle$.
We can use the Hadamard gate to do this.
2. Apply a CNOT take with the first qubit as the control and the second qubit as the target.

@[solution]({
    "id": "preparing_bell_state_solution",
    "codePath": "./Solution.qs"
})
