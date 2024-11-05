First, you measure each of the qubits in the input array, convert the measurement results into integers
and store them in variables `m1` and `m2`. You can decode these results like this:  

- `m1` is $\ket{0}$ and `m2` is $\ket{0}$: the return value is $0\cdot2+0 = 0$
- `m1` is $\ket{0}$ and `m2` is $\ket{1}$: the return value is $0\cdot2+1 = 1$
- `m1` is $\ket{1}$ and `m2` is $\ket{0}$: the return value is $1\cdot2+0 = 2$
- `m1` is $\ket{1}$ and `m2` is $\ket{1}$: the return value is $1\cdot2+1 = 3$

In other words, you treat the measurement results as the binary notation of the return value in big-endian notation.

@[solution]({
"id": "multi_qubit_measurements__full_measurements_solution",
"codePath": "Solution.qs"
})

You can generalize this code to read out an integer in big-endian notation from a qubit array of arbitrary length using several library operations and functions:

- `MeasureEachZ` measures each of the qubits in the array in the computational basis and returns an array of `Result` data type.
- `Reversed` reverses the given array.
- `ResultArrayAsInt` converts an array of bits given as `Result` to an integer, assuming little-endian notation (that's why you have to reverse the array before converting it).

@[solution]({
"id": "multi_qubit_measurements__full_measurements_solution_alt",
"codePath": "SolutionAlt.qs"
})
