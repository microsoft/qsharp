Any multi-qubit state can be prepared from the $|0...0\rangle$ state using an appropriate combination of quantum gates. 
However, sometimes it is easier and more efficient to prepare a state using partial measurements. 
You could prepare a simpler state involving additional qubits, which, when measured, result in a collapse of the remaining qubits to the desired state with a high probability. This is called **post-selection**, and is particularly useful if it is easier to prepare the pre-measurement state with the extra qubits than to prepare the desired state directly using unitary gates alone. This is demonstrated by the following exercise.

### <span style="color:blue">Exercise 8</span>: State preparation using partial measurements

**Input:** Two qubits (in an array) which are in the state $\ket{00}$.

**Goal:**  Modify the qubits to the state $\frac{1}{\sqrt{3}} \big(|00\rangle + |01\rangle + |10\rangle\big)$ using post-selection.

<br/>
<details>
  <summary><b>Need a hint? Click here</b></summary>
  Consider a 3-qubit state $\frac{1}{2}(|00\rangle + |01\rangle + |11\rangle) \otimes |0\rangle + \frac{1}{2} |11\rangle \otimes |1\rangle$.
  What happens when one measures the third qubit?
</details>
