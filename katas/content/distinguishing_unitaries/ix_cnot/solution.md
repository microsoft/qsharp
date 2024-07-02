Let's consider the effect of these gates on the basis states:

<table>
  <tr>
    <th style="text-align:center">State</th>
    <th style="text-align:center">$I \otimes X$</th>
    <th style="text-align:center">$CNOT$</th>    
  </tr>
  <tr>
    <td style="text-align:center">$\ket{00}$</td>
    <td style="text-align:center">$\ket{01}$</td>
    <td style="text-align:center">$\ket{00}$</td>
  </tr>
  <tr>
    <td style="text-align:center">$\ket{01}$</td>
    <td style="text-align:center">$\ket{00}$</td>
    <td style="text-align:center">$\ket{01}$</td>
  </tr>
  <tr>
    <td style="text-align:center">$\ket{10}$</td>
    <td style="text-align:center">$\ket{11}$</td>
    <td style="text-align:center">$\ket{11}$</td>
  </tr>
  <tr>
    <td style="text-align:center">$\ket{11}$</td>
    <td style="text-align:center">$\ket{10}$</td>
    <td style="text-align:center">$\ket{10}$</td>
  </tr>
</table>

We can see that applying these two gates to states with the first qubit in the $\ket{1}$ state yields identical results, but applying them to states with the first qubit in the $\ket{0}$ state produces states that differ in the second qubuit.
This makes sense, since the $CNOT$ gate is defined as "apply $X$ gate to the target qubit if the control qubit is in the $\ket{1}$ state, and do nothing if it is in the $\ket{0}$ state".

Thus, the easiest solution is: allocate two qubits in the $\ket{00}$ state and apply the unitary to them, then measure the second qubit; if it is `One`, the gate is $I \otimes X$, otherwise it's $CNOT$.

@[solution]({
    "id": "distinguishing_unitaries__ix_cnot_solution",
    "codePath": "Solution.qs"
})
