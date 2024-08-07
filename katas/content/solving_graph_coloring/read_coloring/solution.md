The solution to this exercise consists of two parts:

1. Read an integer from an array of qubits of length `nBits`.
   Since you are guaranteed that the qubits in the array are in a basis state, simply measuring them will give you the necessary information and leave the state of the qubits unchanged. 
   You can use the library operation `MeasureEachZ` to measure each qubit in the array without resetting it to the $\ket{0}$ state afterwards. 
   Then, you can convert the array of measurement results to an integer using the function `ResultArrayAsInt`.
   Note that this function does the conversion from a little endian binary encoding, and the exercise asks for the colors to be represented in big endian, so you need to reverse the array of measurement results using the function `Reversed`.
2. Split the given array of qubits into chunks of length `nBits` and read an integer from each of them.
   A convenient library function `Chunks` does exactly that, splitting the array into chunks of the given length.
   Finally, you can apply the integer readout operation to each chunk using the operation `ForEach`.

@[solution]({
    "id": "solving_graph_coloring__read_coloring_solution",
    "codePath": "Solution.qs"
})
