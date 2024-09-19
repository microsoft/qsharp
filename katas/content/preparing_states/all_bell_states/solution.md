> The [Bell states](https://en.wikipedia.org/wiki/Bell_state) form an orthonormal basis in the 4-dimensional space that describes the states of a 2-qubit system. 
You can check that the norm of each of these states is 1, and their inner product of each pair of states is 0.

The goal is to transform the $\ket{00}$ basis state into one of the Bell basis states, depending on the value of `index` given as an input.

This exercise provides two solutions, one of which is based on the previous task, and the second one helps you understand the unitary transformation that converts the computational basis into the Bell basis.

#### Solution 1

Let's use the first Bell state you prepared in the previous task and transform it according to the value of `index`.

$$\frac{1}{\sqrt2} \big(\ket{00} + \ket{11}\big)$$

What transformation do you need to apply to get to the final state?

* If `index = 0`, you do nothing - the prepared state is already $\ket{\Phi^{+}}$.

* If `index = 1`, you need to add a relative phase of $-1$ to the $\ket{11}$ term. Remember that $Z$ gate does exactly that with a qubit:
  
  $$Z(H\ket{0}) \otimes \ket{0} = \frac{1}{\sqrt2} \big(\ket{0} - \ket{1}\big) \otimes \ket{0} = \frac{1}{\sqrt2} \big(\ket{00} - \ket{10}\big)$$
  
  If you now apply the $CNOT$ as before, you'll have:

  $$\frac{1}{\sqrt2} \big(\ket{00} - \ket{\overset{\curvearrowright}{10}}\big) \underset{\text{CNOT}}{\Longrightarrow} \frac{1}{\sqrt2} \big(\ket{00} - \ket{11}\big) = \ket{\Phi^{-}}$$

* If `index = 2`, you need to change the second qubit in both $\ket{00}$ and $\ket{11}$ terms, which can be done applying an $X$ gate:
  
  $$H\ket{0} \otimes X\ket{0} = H\ket{0} \otimes \ket{1} = \frac{1}{\sqrt2} \big(\ket{0} + \ket{1}\big) \otimes \ket{1} = \frac{1}{\sqrt2} \big(\ket{01} + \ket{11}\big)$$
  
  If you now apply the $CNOT$ as before, you'll have:
  
  $$\frac{1}{\sqrt2} \big(\ket{01} + \ket{\overset{\curvearrowright}{11}}\big) \underset{\text{CNOT}}{\Longrightarrow} \frac{1}{\sqrt2} \big(\ket{01} + \ket{10}\big) = \ket{\Psi^{+}}$$

* If `index = 3`, you use the same logic to realize that you need to apply both the $Z$ and $X$ corrections to get $\ket{\Psi^{-}}$ state.

The final sequence of steps is as follows:
1. Apply the $H$ gate to the first qubit. 
2. Apply the $Z$ gate to the first qubit if `index == 1` or `index == 3`.
3. Apply the $X$ gate to the second qubit if `index == 2` or `index == 3`.
4. Apply the $CNOT$ gate with the first qubit as control and the second qubit as target.

@[solution]({
    "id": "preparing_states__all_bell_states_solution_a",
    "codePath": "./SolutionA.qs"
})

#### Solution 2

Let's take a closer look at the unitary transformation $\text{CNOT}\cdot(H \otimes I)$ discussed in the previous task.

$$\frac{1}{\sqrt2} \begin{bmatrix} 1 & 0 & 1 & 0 \\ 0 & 1 & 0 & 1 \\ 0 & 1 & 0 & -1 \\ \underset{\ket{\Phi^{+}}}{\underbrace{1}} & \underset{\ket{\Psi^{+}}}{\underbrace{0}} & \underset{\ket{\Phi^{-}}}{\underbrace{-1}} & \underset{\ket{\Psi^{-}}}{\underbrace{0}} \end{bmatrix}$$


Notice that each of the columns in the unitary matrix corresponds to one of the Bell states.
This unitary transformation transforms the computational basis into the Bell basis, which is exactly what the task asks you to do.

You see that this transformation converts $\ket{00}$ into the first Bell state, $\ket{01}$ into the second Bell state, etc. 
You just need to make sure you set the qubits to the correct state before applying this transformation, using $X$ gates to change the initial $\ket{0}$ states to $\ket{1}$ if needed. 

In Q#, you can use the <a href="https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.convert/intasboolarray">IntAsBoolArray</a> function to convert the input `index` to the right bit pattern.

@[solution]({
    "id": "preparing_states__all_bell_states_solution_b",
    "codePath": "./SolutionB.qs"
})
