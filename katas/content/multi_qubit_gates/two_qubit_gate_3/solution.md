A visual comparison of the two states easily reveals that the amplitudes of the $|01\rangle$ and the $|01\rangle$ components of the state have been swapped. This suggests that we might look for a swap gate that operates on 2 qubits, by changing the components of the 2 qubits to which the amplitudes are 'attached'.

Let's investigate the first possibility. There is a swap gate that might fit the bill; its matrix representation is:
$$
SWAP = 
 \begin{bmatrix} 1 & 0 & 0 & 0\\\ 0 & 0 & 1 & 0\\\ 0 & 1 & 0 & 0\\\ 0 & 0 & 0 & 1\\\ \end{bmatrix}
 $$
 
 and our input state vector is:
$$
\begin{bmatrix} \alpha\\\ \beta\\\ \gamma\\\ \delta\\\ \end{bmatrix}$$
So operating on our input state vector with the SWAP gate gives us
$$
 \begin{bmatrix}1 & 0 & 0 & 0\\\ 0 & 0 & 1 & 0\\\ 0 & 1 & 0 & 0\\\ 0 & 0 & 0 & 1\\\ \end{bmatrix}
\begin{bmatrix} \alpha\\\ \color{blue}\beta\\\ \color{blue}\gamma\\\ \delta\\\ \end{bmatrix}=
\begin{bmatrix} \alpha\\\ \color{red}\gamma\\\ \color{red}\beta\\\ \delta\\\ \end{bmatrix}=
|00\rangle + {\color{red}\gamma} |01\rangle + {\color{red}\beta} |10\rangle + \delta |11\rangle
$$
and we can confirm this with the task solution:



@[solution]({
"id": "multi_qubit_gates__two_qubit_gate_3_solution_a",
"codePath": "./SolutionA.qs"
})
> If you run this solution a few times you might see an apparent anomaly. The test harness uses an input state that has positive values of $\alpha$ and $\delta$ and negative values of $\beta$ and $\gamma$, while
the "actual state" reported (the state prepared by your solution) can come out with negative values of $\alpha$ and $\delta$ and positive values of $\beta$ and $\gamma$. 
We have seen this before in the previous tasks: we can write the apparently anomalous state as $(-1)(\alpha|00\rangle + \beta |01\rangle + \gamma |10\rangle + \delta |11\rangle)$ and see that it differs from the goal state by a global phase of $\pi$ (remember that $e^{i\pi}=-1$). This doesn't mean that your implementation introduced this phase; sometimes the full state simulator used in the test harness produces a global phase in its calculations.
Let's now  follow the hint in the question and try to express the solution using several (possibly controlled) Pauli gates.

If we look at the available controlled gates, CR and its special case CZ produce rotations, and that's not really what we want. So perhaps we are being pointed towards CNOT? If we carefully compare the input with the goal state, we see that the bits in the two basis states of the two qubits are being flipped, which results in a swap. What we need to do is to turn $|01\rangle$ into $|10\rangle$ and $|10\rangle$ into $|01\rangle$ while leaving the other two basis states unchanged.

With some experimentation with sequences of CNOT gates we can arrive at the following sequence of transformations:

<table>
  <col width="150"/>
  <col width="150"/>
  <col width="150"/>
  <col width="150"/>
  <tr>
    <th style="text-align:center">Starting state</th>
    <th style="text-align:center">After CNOT$_{01}$</th>
    <th style="text-align:center">After CNOT$_{10}$</th>
    <th style="text-align:center">After CNOT$_{01}$</th>
  </tr>
  <tr>
    <td style="text-align:center">$|00\rangle$</td>
    <td style="text-align:center">$|00\rangle$</td>
    <td style="text-align:center">$|00\rangle$</td>
    <td style="text-align:center">$|00\rangle$</td>
  </tr>
  <tr>
    <td style="text-align:center">$|01\rangle$</td>
    <td style="text-align:center">$|01\rangle$</td>
    <td style="text-align:center">$|11\rangle$</td>
    <td style="text-align:center">$|10\rangle$</td>
  </tr>
  <tr>
    <td style="text-align:center">$|10\rangle$</td>
    <td style="text-align:center">$|11\rangle$</td>
    <td style="text-align:center">$|01\rangle$</td>
    <td style="text-align:center">$|01\rangle$</td>
  </tr>
  <tr>
    <td style="text-align:center">$|11\rangle$</td>
    <td style="text-align:center">$|10\rangle$</td>
    <td style="text-align:center">$|10\rangle$</td>
    <td style="text-align:center">$|11\rangle$</td>
  </tr>
</table>


@[solution]({
"id": "multi_qubit_gates__two_qubit_gate_3_solution_b",
"codePath": "./SolutionB.qs"
})