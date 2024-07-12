**Inputs:**

1. $N$ qubits in an arbitrary state $\ket{x}$ (input/query register).
2. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).
3. An array of tuples `clause` which describes one clause of a SAT problem instance $clause(x)$.
   Each tuple is an `(Int, Bool)` pair describing one component of the clause:
   - the first element is the index $j$ of the variable $x_j$, 
   - the second element is `true` if the variable is included as itself ($x_j$) and `false` if it is included as a negation ($\neg x_j$).

For example, clause $x_0 \vee \neg x_1$ can be represented as `[(0, true), (1, false)]`.

**Goal:**
Implement a quantum oracle which evaluates the clause $clause(x)$.
    
Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.