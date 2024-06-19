Here we can leverage the same method as used in the previous exercise: find a transformation that maps the basis states to the given states and apply its adjoint to the input state before measuring.

It is a lot harder to recognize the necessary transformation, though. The coefficient $\frac{1}{2}$ hints that there are still two $H$ gates involved, but this transformation is not a tensor product. After some experimentation you can find that the given states can be prepared by applying the $H$ gate to the second qubit of the matching Bell states:

<table>
  <tr>
    <th>Basis state</th>
    <th>Bell state</th>
    <th>State after applying $H$ gate</th>
    <th>Return value</th>
  </tr>
  <tr>
    <td>$\ket{00}$</td>
    <td>$\ket{\Phi^{+}} = \frac{1}{\sqrt{2}} \big (\ket{00} + \ket{11}\big)$</td>
    <td>$\frac{1}{2} \big (\ket{00} + \ket{01} + \ket{10} - \ket{11}\big) = -\ket{S_3}$</td>
    <td>3</td>
  </tr>
  <tr>
    <td>$\ket{10}$ </td>
    <td>$\ket{\Phi^{-}} = \frac{1}{\sqrt{2}} \big (\ket{00} - \ket{11}\big)$</td>
    <td>$\frac{1}{2} \big (\ket{00} + \ket{01} - \ket{10} + \ket{11}\big) = -\ket{S_2}$</td>
    <td>2</td>
  </tr>
  <tr>
    <td>$\ket{01}$</td>
    <td>$\ket{\Psi^{+}} = \frac{1}{\sqrt{2}} \big (\ket{01} + \ket{10}\big)$</td>
    <td>$\frac{1}{2} \big (\ket{00} - \ket{01} + \ket{10} + \ket{11}\big) = -\ket{S_1}$</td>
    <td>1</td>
  </tr>
  <tr>
    <td>$\ket{11}$</td>
    <td>$\ket{\Psi^{-}} = \frac{1}{\sqrt{2}} \big (\ket{01} - \ket{10}\big)$</td>
    <td>$\frac{1}{2} \big (\ket{00} - \ket{01} - \ket{10} - \ket{11}\big) = \ket{S_0}$</td>
    <td>0</td>
  </tr>
</table>

@[solution]({
    "id": "distinguishing_states__four_orthogonal_two_qubit_part_two_solution",
    "codePath": "Solution.qs"
})
