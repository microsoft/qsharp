**Inputs:** 

1. Two qubits in the $|00\rangle$ state (stored in an array of length 2).
2. An integer index.

**Goal:**  Change the state of the qubits to one of the Bell states, based on the value of index:

<table>
  <tr>
    <th style="text-align:center">Index</th>
    <th style="text-align:center">State</th>
  </tr>
  <tr>
    <td style="text-align:center">0</td>
    <td style="text-align:center">$|\Phi^{+}\rangle = \frac{1}{\sqrt{2}} \big (|00\rangle + |11\rangle\big)$</td>
  </tr>
  <tr>
    <td style="text-align:center">1</td>
    <td style="text-align:center">$|\Phi^{-}\rangle = \frac{1}{\sqrt{2}} \big (|00\rangle - |11\rangle\big)$</td>
  </tr>
  <tr>
    <td style="text-align:center">2</td>
    <td style="text-align:center">$|\Psi^{+}\rangle = \frac{1}{\sqrt{2}} \big (|01\rangle + |10\rangle\big)$</td>
  </tr>
  <tr>
    <td style="text-align:center">3</td>
    <td style="text-align:center">$|\Psi^{-}\rangle = \frac{1}{\sqrt{2}} \big (|01\rangle - |10\rangle\big)$</td>
  </tr>
</table>
