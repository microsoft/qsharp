If the qubits are entangled in one of the Bell states, you can't simply measure individual qubits to distinguish the states: if you do, the first two states will both give you 00 or 11, and the last two: 01 or 10. We need to come up with a way to transform the original states to states that are easy to distinguish before measuring them.

First, let's take a look at the preparation of the Bell states starting with the $\ket{00}$ basis state.

> A more detailed discussion of preparing all the Bell states can be found in the task "All Bell States" of the Preparing Quantum States kata.

The unitary transformation $\text{CNOT}\cdot(H \otimes I)$ (which corresponds to applying the $H$ gate to the first qubit, followed by applying the $\text{CNOT}$ gate with the first qubit as control and the second qubit as target) transforms the 4 basis vectors of the computational basis into the 4 Bell states.

$$\text{CNOT}\cdot(H \otimes I) = \frac{1}{\sqrt2} \begin{bmatrix} 1 & 0 & 1 & 0 \\ 0 & 1 & 0 & 1 \\ 0 & 1 & 0 & -1 \\ \underset{\ket{\Phi^{+}}}{\underbrace{1}} & \underset{\ket{\Psi^{+}}}{\underbrace{0}} & \underset{\ket{\Phi^{-}}}{\underbrace{-1}} & \underset{\ket{\Psi^{-}}}{\underbrace{0}} \end{bmatrix}$$

To transform the Bell states back to the basis states, you can apply adjoint of this transformation, which will undo its effects. In this case, both gates used are self-adjoint, so the adjoint transformation will require applying the same gates in reverse order (first $\text{CNOT}$, then $H$).

After this the original states will be transformed as follows:

<table>
      <tr>
        <th>Return value</th>
        <th>Original state</th>
        <th>Maps to basis state</th>
      </tr>
      <tr>
        <td>0</td>
        <td>$\ket{\Phi^{+}} = \frac{1}{\sqrt{2}} \big (\ket{00} + \ket{11}\big)$</td>
        <td>$\ket{00}$</td>
      </tr>
      <tr>
        <td>1</td>
        <td>$\ket{\Phi^{-}} = \frac{1}{\sqrt{2}} \big (\ket{00} - \ket{11}\big)$</td>
        <td>$\ket{10}$</td>
      </tr>
      <tr>
        <td>2</td>
        <td>$\ket{\Psi^{+}} = \frac{1}{\sqrt{2}} \big (\ket{01} + \ket{10}\big)$</td>
        <td>$\ket{01}$</td>
      </tr>
      <tr>
        <td>3</td>
        <td>$\ket{\Psi^{-}} = \frac{1}{\sqrt{2}} \big (\ket{01} - \ket{10}\big)$</td>
        <td>$\ket{11}$</td>
      </tr>
    </table>

These are the same four two-qubit basis states we've seen in the task "Distinguish Four Basis States", though in a different order compared to that task, so mapping the measurement results to the return values will differ slightly.

@[solution]({
    "id": "distinguishing_states__four_bell_states_solution",
    "codePath": "Solution.qs"
})
