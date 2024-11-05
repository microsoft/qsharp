**Inputs:** 

1. Two qubits in the $\ket{00}$ state (stored in an array of length 2).
2. An integer index.

**Goal:**  Change the state of the qubits to one of the Bell states, based on the value of index:

<table>
  <tr>
    <th style="text-align:center">Index</th>
    <th style="text-align:center">State</th>
  </tr>
  <tr>
    <td style="text-align:center">0</td>
    <td style="text-align:center">$\ket{\Phi^{+}} = \frac{1}{\sqrt{2}} \big (\ket{00} + \ket{11}\big)$</td>
  </tr>
  <tr>
    <td style="text-align:center">1</td>
    <td style="text-align:center">$\ket{\Phi^{-}} = \frac{1}{\sqrt{2}} \big (\ket{00} - \ket{11}\big)$</td>
  </tr>
  <tr>
    <td style="text-align:center">2</td>
    <td style="text-align:center">$\ket{\Psi^{+}} = \frac{1}{\sqrt{2}} \big (\ket{01} + \ket{10}\big)$</td>
  </tr>
  <tr>
    <td style="text-align:center">3</td>
    <td style="text-align:center">$\ket{\Psi^{-}} = \frac{1}{\sqrt{2}} \big (\ket{01} - \ket{10}\big)$</td>
  </tr>
</table>
