To solve this exercise, you need to follow the QPE algorithm as described in this lesson.

1. Allocate two qubit registers, $n$-qubit `phaseRegister` and single-qubit `eigenstate` (this one can be just an individual qubit rather than a qubit array for simplicity).
2. Prepare the initial state of the algorithm: `eigenstate` in the state $\ket{\psi}$ using the unitary $P$ and `phaseRegister` in the state that is an even superposition of all basis states using Hadamard gates.
3. Apply the controlled $U$ gates using two `for` loops. The outer loop will iterate over the qubits of `phaseRegister`, from the first qubit storing the least significant digit to the last one storing the most significant one. The inner loop will apply controlled $U$ $2^k$ times, where $k$ is the variable used as the outer loop counter.
4. Apply the adjoint QFT. Here, you can use the library operation `ApplyQFT`, keeping in mind that it applies only the rotations part of the quantum Fourier transform. To implement the complete transform using this operation, you need to reverse the order of qubits in the register after applying `ApplyQFT`. Since here you need to use adjoint QFT, you need to reverse the order of qubits before calling `Adjoint ApplyQFT`.
5. Measure the qubits in `phaseRegister` and convert the result to an integer. `MeasureInteger` does exactly that for a register in little endian. You also need to reset the qubit used as the eigenstate before releasing it.

@[solution]({
    "id": "phase_estimation__implement_qpe_solution", 
    "codePath": "Solution.qs"
})
