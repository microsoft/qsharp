Let’s look at some examples of basis states to illustrate the binary numbering system. 

The 4 basis states on $N = 2$ qubits can be split in two columns, where the left column represents the basis states that form the required superposition state for `isEven = true` and the right column - the basis states that form the required superposition state for `isEven = false`.

<table>
  <tr><th>even</th><th>odd</th></tr>
  <tr><td> <b>0</b>0 </td><td> <b>0</b>1 </td></tr>
  <tr><td> <b>1</b>0 </td><td> <b>1</b>1 </td></tr>
</table>
 
If you do the same basis state split for $N = 3$ qubits, the pattern becomes more obvious.

<table>
  <tr><th>even</th><th>odd</th></tr>
  <tr><td> <b>00</b>0 </td><td> <b>00</b>1</td></tr>
  <tr><td> <b>01</b>0 </td><td> <b>01</b>1</td></tr>
  <tr><td> <b>10</b>0 </td><td> <b>10</b>1</td></tr>
  <tr><td> <b>11</b>0 </td><td> <b>11</b>1</td></tr>
</table>

The two leftmost qubits go through all possible basis states for `isEven = true` and for `isEven = false`, and the rightmost qubit stays in the $\ket{0}$ state for `isEven = true` and in the $\ket{1}$ state for `isEven = false`. 

A quick sanity check for $N = 4$ qubits re-confirms the pattern.

<table>
  <tr>
    <th>even</th>
    <th>odd</th>
  </tr>
  <tr><td> <b>000</b>0 </td><td> <b>000</b>1</td></tr>
  <tr><td> <b>001</b>0 </td><td> <b>001</b>1</td></tr>
  <tr><td> <b>010</b>0 </td><td> <b>010</b>1</td></tr>
  <tr><td> <b>011</b>0 </td><td> <b>011</b>1</td></tr>
  <tr><td> <b>100</b>0 </td><td> <b>100</b>1</td></tr>
  <tr><td> <b>101</b>0 </td><td> <b>101</b>1</td></tr>
  <tr><td> <b>110</b>0 </td><td> <b>110</b>1</td></tr>
  <tr><td> <b>111</b>0 </td><td> <b>111</b>1</td></tr>
</table>
 
Again, the three leftmost qubits go through all possible basis states in both columns, and the rightmost qubit stays in the same state in each column. 

The solution is to put all qubits except the rightmost one into an equal superposition (similar to what you did in the 'Superposition of all basis vectors' task) and to set the rightmost qubit to $\ket{0}$ or $\ket{1}$ depending on the `isEven` flag, using the $X$ operator to convert $\ket{0}$ to $\ket{1}$ if `isEven = false`.
 

@[solution]({
    "id": "preparing_states__even_odd_solution",
    "codePath": "./Solution.qs"
})
