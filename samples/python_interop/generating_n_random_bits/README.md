# A simple Q# program to generate an array of N random bits and Python program to count the number of 1s returned in the result array

## Description

In this example, a Q# program generates an array of random bits based on a user-defined maximum number (`max`), which determines the range for random bit generation. The program calculates the necessary number of qubits using the `BitSizeI()` function and creates a random bit sequence by applying an $H$ gate to the qubits. The resulting array of bits is then processed by a Python program that counts the number of `1`s in the array and displays both the count and the array itself.
