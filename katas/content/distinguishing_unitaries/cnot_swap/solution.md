Again, let's consider the effect of these gates on the basis states:

<table>
  <tr>
    <th style="text-align:center">State</th>
    <th style="text-align:center">$CNOT_{12}$</th>
    <th style="text-align:center">$SWAP$</th>    
  </tr>
  <tr>
    <td style="text-align:center">$\ket{00}$</td>
    <td style="text-align:center">$\ket{00}$</td>
    <td style="text-align:center">$\ket{00}$</td>
  </tr>
  <tr>
    <td style="text-align:center">$\ket{01}$</td>
    <td style="text-align:center">$\ket{01}$</td>
    <td style="text-align:center">$\ket{10}$</td>
  </tr>
  <tr>
    <td style="text-align:center">$\ket{10}$</td>
    <td style="text-align:center">$\ket{11}$</td>
    <td style="text-align:center">$\ket{01}$</td>
  </tr>
  <tr>
    <td style="text-align:center">$\ket{11}$</td>
    <td style="text-align:center">$\ket{10}$</td>
    <td style="text-align:center">$\ket{11}$</td>
  </tr>
</table>

Same as in the previous task, applying these two gates to any basis state other than $\ket{00}$ yields different results, and we can use any of these states to distinguish the unitaries.

The easiest solution is: prepare two qubits in the $\ket{01}$ state and apply the unitary to them, then measure the first qubit; if it is still `Zero`, the gate is $CNOT_{12}$, otherwise it's $SWAP$. Remember that this time the second qubit might end up in $\ket{1}$ state, so it needs to be reset to $\ket{0}$ before releasing it.

@[solution]({
    "id": "distinguishing_unitaries__cnot_swap_solution",
    "codePath": "Solution.qs"
})
