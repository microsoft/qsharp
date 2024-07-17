**Inputs:**

1. $N$ qubits in an arbitrary state $\ket{x}$ (input/query register).
2. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).
3. A two-dimensional array of tuples `formula` which describes a SAT problem instance $f(x)$.
   $i$-th element of `formula` is an array of tuples that describes the $i$-th clause of $f(x)$ using the same format as the previous exercise. Each clause of the formula is guaranteed to have exactly three literals.

For example, a three-variable SAT formula with one clause $f(x) = (x_0 \vee x_1 \vee x_2)$ can be represented as `[[(0, true), (1, true), (2, true)]]`,
and its solutions will be `(true, false, false)`, `(false, true, false)` and `(false, false, true)`.

**Goal:**
Implement a quantum oracle which evaluates the exactly-1 3-SAT formula $f(x)$.
    
Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.