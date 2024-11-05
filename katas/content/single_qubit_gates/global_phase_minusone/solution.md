A global phase is a phase factor that multiplies the entire quantum state. It isn't observable when measuring the qubit's state, as the probabilities remain unchanged. However, it's significant when considering quantum state transformations.

Your task is to implement an operation that transforms the given qubit state from $\ket{\psi} = \beta \ket{0} + \gamma \ket{1}$ to $- \beta \ket{0} - \gamma \ket{1}$.

To do that, you use a sequence of gates. The Pauli Z gate followed by the Pauli X gate can be used to achieve this effect when applied in succession twice.

1. **Apply the Pauli Z gate**: The Z gate multiplies the $\ket{1}$ state by $-1$ but doen't change the $\ket{0}$ state, converting the state $\beta \ket{0} + \gamma \ket{1}$ to $\beta \ket{0} - \gamma \ket{1}$.

   The matrix representation of the Z gate is:

   $$
   Z =
   \begin{bmatrix}1 & 0 \\ 0 & -1 \end{bmatrix}
   $$

2. **Apply the Pauli X gate**: The X gate flips the $\ket{0}$ and $\ket{1}$ basis states, converting $\beta \ket{0} - \gamma \ket{1}$ state to $\beta \ket{1} - \gamma \ket{0}$.

   The matrix representation of the X gate is:

   $$
   X =
   \begin{bmatrix}0 & 1 \\ 1 & 0\end{bmatrix}
   $$

3. **Repeat the Z and X gates**: Applying the Z gate again will multiply the $\ket{1}$ state (that used to be $\ket{0}$), converting the state $\beta \ket{1} - \gamma \ket{0}$ to $- \beta \ket{1} - \gamma \ket{0}$. Finally, the second X gate will restore the original basis states, but now with both amplitudes having acquired an additional phase of $-1$. This means the state has been multiplied by $-1$, achieving the required global phase change.

@[solution]({
"id": "single_qubit_gates__global_phase_minusone_solution",
"codePath": "./Solution.qs"
})
