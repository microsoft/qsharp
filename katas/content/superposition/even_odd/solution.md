### Solution

Let’s look at some examples of basis states to illustrate the binary numbering system. 

The 4 basis states on $N = 2$ qubits can be split in two columns, where the left column represents the basis states that form the required superposition state for `isEven = true` and the right column - the basis states that form the required superposition state for `isEven = false`.

| even     | odd     |
| -------- | ------- |
| **0**0   | **0**1  |
| **1**0   | **1**1  |
 
If we do the same basis state split for $N = 3$ qubits, the pattern becomes more obvious.

| even     | odd     |
| -------- | ------- |
| **00**0  | **00**1 |
| **01**0  | **01**1 |
| **10**0  | **10**1 |
| **11**0  | **11**1 |

The two leftmost qubits go through all possible basis states for `isEven = true` and for `isEven = false`, and the rightmost qubit stays in the $|0\rangle$ state for `isEven = true` and in the $|1\rangle$ state for `isEven = false`. 

A quick sanity check for $N = 4$ qubits re-confirms the pattern.

| even      | odd      |
| --------- | -------- |
| **000**0  | **000**1 |
| **001**0  | **001**1 |
| **010**0  | **010**1 |
| **011**0  | **011**1 |
| **100**0  | **100**1 |
| **101**0  | **101**1 |
| **110**0  | **110**1 |
| **111**0  | **111**1 |
 
Again, the three leftmost qubits go through all possible basis states in both columns, and the rightmost qubit stays in the same state in each column. 

The solution is to put all qubits except the rightmost one into an equal superposition (similar to what we did in the 'Superposition of all basis vectors' task) and to set the rightmost qubit to $|0\rangle$ or $|1\rangle$ depending on the `isEven` flag, using the X operator to convert $|0\rangle$ to $|1\rangle$ if `isEven = false`.
 

@[solution]({
    "id": "superposition__even_odd_solution",
    "codePath": "./Solution.qs"
})
