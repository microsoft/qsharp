Let's start by implementing a helper operation that we'll need to count the number of $1$ bits in the bit string. 
This function will act on a register of $m$ qubits that store an integer and increment this integer modulo $2^m$. We'll call it `Increment`, and store the input in little endian notation.

Consider the logic of incrementing a classical number in binary notation:

* First, we flip the least significant bit.
* If the least significant bit is now $0$, increment the rest of the number (i.e., flip the second-least significant bit, check whether it is $0$, and apply the same procedure recursively). 

We can swap these two steps, so that we would first increment the rest of the number if the least significant bit is $1$, and then flip the least significant bit.

How can we translate these steps to quantum gates?

* Classical conditional statements are converted to controlled gates (remember that all computations have to be done without measurements, so that if the input register is in superposition, it remains in superposition after the computation as well). We can use the `Controlled` functor to call controlled version of `Increment`, with the first argument being the array of control qubits (the least significant bit) and the second argument being the arguments passed to the `Increment` (the rest of the bits).
* Flipping the least significant bit corresponds to applying an $X$ gate to it.

With this helper operation implemented, solving the task becomes clear:

1. Allocate the auxiliary qubits to store the number of $1$ bits in the bit string. The necessary number of qubits is the number of digits in the binary notation of the bit string length $N$, which you can calculate using a conveninent Q# library function `BitSizeI`.
2. Count the number of $1$ bits in the input register. To do this, you need to increment the counter array for each qubit of the input register that is in $\ket{1}$ state using a controlled variant of the `Increment` operation with that qubit as control.
3. Check whether the number of $1$ bits is exactly $\frac{N}{2}$, and flip the target qubit if it is using `ApplyControlledOnInt` operation.
4. As usual, uncompute the changes done to the auxiliary qubits before releasing them.

@[solution]({
    "id": "marking_oracles__balanced_solution",
    "codePath": "./Solution.qs"
})
