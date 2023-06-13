## <span style="color:blue">Exercise 3</span>: Distinguish $|+\rangle$ and $|-\rangle$ using the `Measure` operation

**Input**: A qubit which is guaranteed to be in either the $\ket +$ state, or the $\ket -$ state.

**Output**: `true` if the qubit is in the $\ket -$ state, or `false` if it was in the $\ket +$ state. 

> To perform a single-qubit measurement in a certain Pauli basis using the `Measure` operation, 
> you need to pass it two parameters: first, an array of one `Pauli` constant (`PauliX`, `PauliY` or `PauliZ`), and second, an array of one qubit you want to measure.
