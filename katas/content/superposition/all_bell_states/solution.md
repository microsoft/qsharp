> The [Bell states](https://en.wikipedia.org/wiki/Bell_state) form an orthonormal basis in the 4-dimensional space that describes the states of a 2-qubit system. 
You can check that the norm of each of these states is 1, and their inner product of each pair of states is 0.

The goal is to transform the $|00\rangle$ basis state into one of the Bell basis states, depending on the value of `index` given as an input.

We will describe two solutions, one of which will be based on the previous task, and the second one will help us understand the unitary transformation that converts the computational basis into the Bell basis.

#### Solution 1

Let's use the first Bell state we prepared in the previous task and transform it according to the value of `index`.

$$\frac{1}{\sqrt2} \big(|00\rangle + |11\rangle\big)$$

What transformation do we need to apply to get to the final state?

* If `index = 0`, we do nothing - the prepared state is already $|\Phi^{+}\rangle$.

* If `index = 1`, we need to add a relative phase of $-1$ to the $|11\rangle$ term. Remember that $Z$ gate does exactly that with a qubit:
  
  $$Z(H|0\rangle) \otimes |0\rangle = \frac{1}{\sqrt2} \big(|0\rangle - |1\rangle\big) \otimes |0\rangle = \frac{1}{\sqrt2} \big(|00\rangle - |10\rangle\big)$$
  
  If we now apply the $CNOT$ as before, we will have:

  $$\frac{1}{\sqrt2} \big(|00\rangle - |\overset{\curvearrowright}{10}\rangle\big) \underset{\text{CNOT}}{\Longrightarrow} \frac{1}{\sqrt2} \big(|00\rangle - |11\rangle\big) = |\Phi^{-}\rangle$$

* If `index = 2`, we need to change the second qubit in both $|00\rangle$ and $|11\rangle$ terms, which can be done applying an $X$ gate:
  
  $$H|0\rangle \otimes X|0\rangle = H|0\rangle \otimes |1\rangle = \frac{1}{\sqrt2} \big(|0\rangle + |1\rangle\big) \otimes |1\rangle = \frac{1}{\sqrt2} \big(|01\rangle + |11\rangle\big)$$
  
  If we now apply the $CNOT$ as before, we will have:
  
  $$\frac{1}{\sqrt2} \big(|01\rangle + |\overset{\curvearrowright}{11}\rangle\big) \underset{\text{CNOT}}{\Longrightarrow} \frac{1}{\sqrt2} \big(|01\rangle + |10\rangle\big) = |\Psi^{+}\rangle$$

* If `index = 3`, we use the same logic to realize that we need to apply both the $Z$ and $X$ corrections to get $|\Psi^{-}\rangle$ state.

The final sequence of steps is as follows:
1. Apply the $H$ gate to the first qubit. 
2. Apply the $Z$ gate to the first qubit if `index == 1` or `index == 3`.
3. Apply the $X$ gate to the second qubit if `index == 2` or `index == 3`.
4. Apply the $CNOT$ gate with the first qubit as control and the second qubit as target.

@[solution]({
    "id": "superposition__all_bell_states_solution_a",
    "codePath": "./SolutionA.qs"
})

#### Solution 2

Let's take a closer look at the unitary transformation $\text{CNOT}\cdot(H \otimes I)$ discussed in task 6 (see equation 6.1).

$$\frac{1}{\sqrt2} \begin{bmatrix} 1 & 0 & 1 & 0 \\\ 0 & 1 & 0 & 1 \\\ 0 & 1 & 0 & -1 \\\ \underset{|\Phi^{+}\rangle}{\underbrace{1}} & \underset{|\Psi^{+}\rangle}{\underbrace{0}} & \underset{|\Phi^{-}\rangle}{\underbrace{-1}} & \underset{|\Psi^{-}\rangle}{\underbrace{0}} \end{bmatrix}$$


Notice that each of the columns in the unitary matrix corresponds to one of the Bell states.
This unitary transformation transforms the computational basis into the Bell basis, which is exactly what the task asks us to do.

We see that this transformation converts $|00\rangle$ into the first Bell state, $|01\rangle$ into the second Bell state, etc. 
We just need to make sure we set the qubits to the correct state before applying this transformation, using $X$ gates to change the initial $|0\rangle$ states to $|1\rangle$ if needed. 

In Q#, we can use the <a href="https://learn.microsoft.com/qsharp/api/qsharp-lang/microsoft.quantum.convert/intasboolarray">IntAsBoolArray</a> function to convert the input `index` to the right bit pattern.

@[solution]({
    "id": "superposition__all_bell_states_solution_b",
    "codePath": "./SolutionB.qs"
})
