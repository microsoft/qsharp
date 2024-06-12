Because all the bit strings are guaranteed to be different and we are not limited in the number of measurements we can do, we can use a simpler solution than in the previous task.

When we measure all qubits of a certain superposition state, it collapses to one of the basis vectors that comprised the superposition. We can do exactly that and compare the resulting state to the given bit strings to see which array it belongs to.

We use the built-in library primitives. First, we measure the quantum register with respect to the standard computational basis, i.e., the eigenbasis of `PauliZ` using `MeasureInteger()` operation, which converts it to an integer.

Next, we convert each of the input bit strings to an integer using the `BoolArrayAsInt()` function. Both these functions use little-endian encoding when converting bits to integers.

Now that we have two integers, we can easily compare the measurement results to each of the bit strings in the first array to check whether they belong to it; if they do, we know we were given the first state, otherwise it was the second state.

@[solution]({
    "id": "distinguishing_states__two_superposition_states_bit_strings_solution",
    "codePath": "Solution.qs"
})
