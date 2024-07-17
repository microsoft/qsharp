**Inputs:**

1. $N$ qubits in an arbitrary state $\ket{x}$ (input/query register).
2. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).
3. A two-dimensional array of tuples `formula` which describes a SAT problem instance $f(x)$.
   $i$-th element of `formula` is an array of tuples that describes the $i$-th clause of $f(x)$ using the same format as the previous exercise. 

For example, a two-variable SAT formula that evaluates the XOR of two inputs $f(x) = (x_0 \vee x_1) \wedge (\neg x_0 \vee \neg x_1)$ can be represented as `[[(0, true), (1, true)], [(0, false), (1, false)]]`.

**Goal:**
Implement a quantum oracle which evaluates the formula $f(x)$.
    
Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.