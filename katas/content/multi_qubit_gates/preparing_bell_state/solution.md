We've seen this state before in the Multi-Qubit Systems kata, where we established that this state is not separable, i.e., it can not be prepared using just the single-qubit gates. To prepare it, we need to use $CNOT$ gate.

Let's look at the effect of the $CNOT$ gate on a separable state described in the tutorial:
$$CNOT_{1,2}\big(\alpha\ket{0} + \beta\ket{1}\big) \otimes \ket{0} = CNOT_{1,2}(\alpha\ket{00} + \beta\ket{10}) = \alpha\ket{00} + \beta\ket{11}$$

This resulting state is exactly the state we need to prepare, with $\alpha = \beta = \frac{1}{\sqrt{2}}$!

The solution takes two steps:
1. Prepare a state $\big(\frac{1}{\sqrt{2}}\ket{0} + \frac{1}{\sqrt{2}}\ket{1}\big) \otimes \ket{0}$.
We can use the Hadamard gate to do this.
2. Apply a $CNOT$ gate with the first qubit as the control and the second qubit as the target.

@[solution]({
    "id": "multi_qubit_gates__preparing_bell_state_solution",
    "codePath": "./Solution.qs"
})
