First, we measure both qubits in the input array and store the result in `m1` and `m2`. We can decode these results like this:  
- `m1` is $|0\rangle$ and `m2` is $|0\rangle$: we return $0\cdot2+0 = 0$
- `m1` is $|0\rangle$ and `m2` is $|1\rangle$: we return $0\cdot2+1 = 1$
- `m1` is $|1\rangle$ and `m2` is $|0\rangle$: we return $1\cdot2+0 = 2$
- `m1` is $|1\rangle$ and `m2` is $|1\rangle$: we return $1\cdot2+1 = 3$

In other words, we treat the measurement results as the binary notation of the return value in big endian notation.

@[solution]({
"id": "full_measurements_solution",
"codePath": "solution.qs"
})

We can generalize this code to read out an integer in big endian notation from a qubit array of arbitrary length using several library operations and functions:

* `MeasureEachZ` measures each of the qubits in the array in the computational basis and returns an array of `Result` data type.
* `Reversed` reverses the given array.
* `ResultArrayAsInt` converts an array of bits given as `Result` to an integer, assuming little-endian notation (that's why we have to reverse the array before converting it).

@[solution]({
"id": "full_measurements_solution_alt",
"codePath": "solution_alt.qs"
})
