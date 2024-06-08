Here we can leverage the same method as used in the previous exercise: find a transformation that maps the basis states to the given states and apply its adjoint to the input state before measuring.

It is a lot harder to recognize the necessary transformation, though. The coefficient $\frac{1}{2}$ hints that there are still two $H$ gates involved, but this transformation is not a tensor product. After some experimentation you can find that the given states can be prepared by applying the $H$ gate to the second qubit of the matching Bell states:

| Basis state | Bell state  | Input state (after applying $H$ gate to the second qubit) | Return value |
|     :---:    |     :---:      |        :---:        |      :---:        |
| $\ket{00}$     | $\ket{\Phi^{+}} = \frac{1}{\sqrt{2}} \big (\ket{00} + \ket{11}\big)$        | $\frac{1}{2} \big (\ket{00} + \ket{01} + \ket{10} - \ket{11}\big) = -\ket{S_3}$      | 3 |
| $\ket{10}$     | $\ket{\Phi^{-}} = \frac{1}{\sqrt{2}} \big (\ket{00} - \ket{11}\big)$        | $\frac{1}{2} \big (\ket{00} + \ket{01} - \ket{10} + \ket{11}\big) = -\ket{S_2}$      | 2 |
| $\ket{01}$     | $\ket{\Psi^{+}} = \frac{1}{\sqrt{2}} \big (\ket{01} + \ket{10}\big)$        | $\frac{1}{2} \big (\ket{00} - \ket{01} + \ket{10} + \ket{11}\big) = -\ket{S_1}$      | 1 |
| $\ket{11}$     | $\ket{\Psi^{-}} = \frac{1}{\sqrt{2}} \big (\ket{01} - \ket{10}\big)$        | $\frac{1}{2} \big (\ket{00} - \ket{01} - \ket{10} - \ket{11}\big) = \ket{S_0}$      | 0 |

@[solution]({
    "id": "distinguishing_states__four_orthogonal_two_qubit_part_two_solution",
    "codePath": "Solution.qs"
})
