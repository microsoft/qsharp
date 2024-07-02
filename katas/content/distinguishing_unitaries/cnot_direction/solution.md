Again, let's consider the effect of these gates on the basis states:

<table>
  <tr>
    <th style="text-align:center">State</th>
    <th style="text-align:center">$CNOT_{12}$</th>
    <th style="text-align:center">$CNOT_{21}$</th>    
  </tr>
  <tr>
    <td style="text-align:center">$\ket{00}$</td>
    <td style="text-align:center">$\ket{00}$</td>
    <td style="text-align:center">$\ket{00}$</td>
  </tr>
  <tr>
    <td style="text-align:center">$\ket{01}$</td>
    <td style="text-align:center">$\ket{01}$</td>
    <td style="text-align:center">$\ket{11}$</td>
  </tr>
  <tr>
    <td style="text-align:center">$\ket{10}$</td>
    <td style="text-align:center">$\ket{11}$</td>
    <td style="text-align:center">$\ket{10}$</td>
  </tr>
  <tr>
    <td style="text-align:center">$\ket{11}$</td>
    <td style="text-align:center">$\ket{10}$</td>
    <td style="text-align:center">$\ket{01}$</td>
  </tr>
</table>

We can see that applying these two gates to any basis state other than $\ket{00}$ yields different results, and we can use any of these states to distinguish the unitaries.

Thus, the easiest solution is: prepare two qubits in the $\ket{01}$ state and apply the unitary to them, then measure the first qubit; if it is still `Zero`, the gate is $CNOT_{12}$, otherwise it's $CNOT_{21}$.

@[solution]({
    "id": "distinguishing_unitaries__cnot_direction_solution",
    "codePath": "Solution.qs"
})
