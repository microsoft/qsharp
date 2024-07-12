# Passing arguments from Python to Q# and processing measurement results in Python

In this example, a Q# program generates an array of random bits based on a user-defined number of bits (`nQubits`), which determines the range for random bit generation. The program creates a random bit sequence by applying an $H$ gate to the qubits. The Python program then processes the resulting array of bits by counting the number of `One`s and displays this count along with the array of random bits and the integer representation of the generated bits.
