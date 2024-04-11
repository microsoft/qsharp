A global phase is a phase factor that multiplies the entire quantum state. It is not observable when measuring the qubit's state, as the probabilities remain unchanged. However, it is significant when considering quantum state transformations.

Our task is to implement an operation that transforms the given qubit state from $|\psi\rangle = \beta |0\rangle + \gamma |1\rangle$ to$- \beta |0\rangle - \gamma |1\rangle$.

To apply a global phase of $\pi$(which is equivalent to multiplying by -1), we utilize a sequence of gates. The Pauli Z gate followed by the Pauli X gate can be used to achieve this effect when applied in succession twice.

### Step-by-Step Solution

1. **Apply the Pauli Z gate**: The Z gate adds a phase of$\pi$to the$\beta |0\rangle + \gamma |1\rangle$ state but does not change the $|0\rangle$ state.

   The matrix representation of the Z gate is:

   $$
   Z =
   \begin{bmatrix}1 & 0 \\\ 0 & -1 \end{bmatrix}
   $$

2. **Apply the Pauli X gate**: The X gate flips the$|0\rangle$ and$|1\rangle$states.

   The matrix representation of the X gate is:

   $$
   X =
   \begin{bmatrix}0 & 1 \\\1 & 0\end{bmatrix}
   $$

3. **Repeat the Z and X gates**: Applying the Z gate again will add another phase of$\pi$ to the now$|0\rangle$ state (since the X gate has flipped the states), and the second X gate will restore the original basis states but with both amplitudes having acquired an additional phase of$\pi$ . This means our state has been multiplied by -1, achieving the global phase change.

@[solution]({
"id": "single_qubit_gates__global_phase_change_solution",
"codePath": "./Solution.qs"
})
