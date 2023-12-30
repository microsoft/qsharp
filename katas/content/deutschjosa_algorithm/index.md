# Deutsch-Jozsa algorithm

@[section]({ "id": "deutsch_joza__overview", "title": "Overview" })

The **Deutsch–Jozsa algorithm** quantum kata is a series of exercises designed to get you familiar with programming in Q#.

It covers the following topics:

* writing oracles (quantum operations which implement certain classical functions),
* Bernstein-Vazirani algorithm for recovering the parameters of a scalar product function,
* Deutsch-Jozsa algorithm for recognizing a function as constant or balanced, and
* writing tests in Q#.

You can read more about the quantum oracles, Deutsch and Deutsch-Jozsa algorithms in the [ExploringDeutschJozsaAlgorithm](https://github.com/microsoft/QuantumKatas/blob/531286588348b322d70d5e053bc60e0be126065e/tutorials/ExploringDeutschJozsaAlgorithm/DeutschJozsaAlgorithmTutorial_P1.ipynb) tutorial.

Each task is wrapped in one operation preceded by the description of the task. Your goal is to fill in the blanks (marked with `// ... comments`) with some Q# code that solves the task. To verify your answer, run the cell with Ctrl+Enter (⌘+Enter on macOS).

@[section]({ "id": "deutsch_joza__part1", "title": "Part I. Oracles" })

In this section you will implement oracles defined by classical functions using the following rules:

* A function $f\left(x_0, ..., x_{N-1}\right)$ with N bits of input $x = \left(x_0, ..., x_{N-1}\right)$ and 1 bit of output $y$  defines an oracle which acts on N input qubits and 1 output qubit.

* The oracle effect on qubits in computational basis states is defined as follows:  ( $|x\rangle |y\rangle \to |x\rangle |y \oplus f(x)\rangle$
 is addition modulo 2 ).
* The oracle effect on qubits in superposition is defined following the linearity of quantum operations.

* The oracle must act properly on qubits in all possible input states.

You can read more about quantum oracles [here](https://docs.microsoft.com/azure/quantum/concepts-oracles).

### Task 1.1. f(x) = 0

**Inputs:**

1. N qubits in an arbitrary state  (input register)
2. a qubit in an arbitrary state (output qubit)

**Goal:** Transform state $|x, y\rangle$  into state $|x, y \oplus f(x)\rangle$  ($\oplus$ is addition modulo 2).

```qsharp
%kata T11_Oracle_Zero 

operation Oracle_Zero (x : Qubit[], y : Qubit) : Unit {
    // Since f(x) = 0 for all values of x, |y ⊕ f(x)⟩ = |y⟩.
    // This means that the operation doesn't need to do any transformation to the inputs.
    
    // Run the cell (using Ctrl/⌘ + Enter) to see that the test passes.
}
```

### Task 1.2. $f(x) = 1$

**Inputs:**

1. N qubits in an arbitrary state $|x\rangle$ (input register).
2. A qubit in an arbitrary state $|y\rangle$ (output qubit).

**Goal:** Transform state $|x, y\rangle$ into state $|x, y \oplus f(x)\rangle$  ($\oplus$ is addition modulo 2).

```qsharp
%kata T12_Oracle_One 

operation Oracle_One (x : Qubit[], y : Qubit) : Unit {
    // ...
}
```

### Task 1.3. $f(x) = x_k$ (the value of k-th qubit)

**Inputs:**

1. N qubits in an arbitrary state $|x\rangle$ (input register).
2. A qubit in an arbitrary state $|y\rangle$  (output qubit)
3. 0-based index of the qubit from input register ($0 \le k < N$)

**Goal:** Transform state $|x, y\rangle$ into state $|x, y \oplus x_k\rangle$
 ($\oplus$  is addition modulo 2).

 ```qsharp
%kata T13_Oracle_Kth_Qubit 

open Microsoft.Quantum.Diagnostics;

operation Oracle_Kth_Qubit (x : Qubit[], y : Qubit, k : Int) : Unit {
    // The following line enforces the constraints on the value of k that you are given.
    // You don't need to modify it. Feel free to remove it, this won't cause your code to fail.
    EqualityFactB(0 <= k and k < Length(x), true, "k should be between 0 and N-1, inclusive");

    // ...
}
 ```

### Task 1.4. f(x) = 1 if x has odd number of 1s, and 0 otherwise

**Inputs:**

1. N qubits in an arbitrary state $|x\rangle$ (input register).
2. a qubit in an arbitrary state $|y\rangle$  (output qubit).

**Goal:** Transform state $|x, y\rangle$ into state $|x, y \oplus f(x)\rangle$ ($\oplus$ is addition modulo 2).

**Hint:**
 $f(x)$ can be represented as $x_0 \oplus x_1 \oplus ... \oplus x_{N-1}$.

```qsharp
%kata T14_Oracle_OddNumberOfOnes

operation Oracle_OddNumberOfOnes (x : Qubit[], y : Qubit) : Unit {
    // ...
}
```

### Task 1.5. $f(x) = \bigoplus\limits_{i=0}^{N-1} r_i x_i$ for a given bit vector r (scalar product function)

**Inputs:**

1. N qubits in an arbitrary state $|x\rangle$ (input register).
2. A qubit in an arbitrary state $|y\rangle$ (output qubit).
3. A bit vector of length N represented as an `Int[]. You are guaranteed that the qubit array and the bit vector have the same length.

**Goal:** Transform state $|x, y\rangle$ into state $|x, y \oplus f(x)\rangle$ ($\oplus$ is addition modulo 2).

```qsharp
%kata T15_Oracle_ProductFunction

open Microsoft.Quantum.Diagnostics;

operation Oracle_ProductFunction (x : Qubit[], y : Qubit, r : Int[]) : Unit {
    // The following line enforces the constraint on the input arrays.
    // You don't need to modify it. Feel free to remove it, this won't cause your code to fail.
    EqualityFactI(Length(x), Length(r), "Arrays should have the same length");

    // ...
}
```

### Task 1.6. $f(x) = \bigoplus\limits_{i=0}^{N-1} \left(r_i x_i + (1 - r_i) (1 - x_i) \right)$ for a given bit vector r (scalar product function)

**Inputs:**

1. N qubits in an arbitrary state $|x\rangle$ (input register).
2. A qubit in an arbitrary state $|y\rangle$ (output qubit).
3. A bit vector of length N represented as an `Int[]. You are guaranteed that the qubit array and the bit vector have the same length.

**Goal:** Transform state  into state $|x, y \oplus f(x)\rangle$ ($\oplus$ is addition modulo 2).

**Hint** Since each addition is done modulo 2, you can evaluate the effect of each term independently.

```qsharp
%kata T16_Oracle_ProductWithNegationFunction

open Microsoft.Quantum.Diagnostics;

operation Oracle_ProductWithNegationFunction (x : Qubit[], y : Qubit, r : Int[]) : Unit {
    // The following line enforces the constraint on the input arrays.
    // You don't need to modify it. Feel free to remove it, this won't cause your code to fail.
    EqualityFactI(Length(x), Length(r), "Arrays should have the same length");

    // ...
}
```

### Task 1.7. $f(x) = \bigoplus\limits_{i=0}^{N-1} x_i +$ (1 if prefix of x is equal to the given bit vector, and 0 otherwise) modulo 2

**Inputs:**

1. N qubits in an arbitrary state $|x\rangle$ (input register).
2. A qubit in an arbitrary state $|y\rangle$ (output qubit).
3. A bit vector of length K represented as an `Int[]` ($1 \le K \le N$).

**Goal:** Transform state $1 \le K \le N$ into state $|x, y \oplus f(x)\rangle$ ($\oplus$ is addition modulo 2).

>A prefix of length K of a state $|x\rangle = |x_0, ..., x_{N-1}\rangle$ is the state of its first K qubits $|x_0, ..., x_{K-1}\rangle$. For example, a prefix of length 2 of a state $|0110\rangle$ is 01.

**Hint** The first term is the same as in task 1.4. To implement the second term, you can use `Controlled` functor which allows to perform multicontrolled gates (gates with multiple control qubits).

```qsharp
%kata T17_Oracle_HammingWithPrefix

open Microsoft.Quantum.Diagnostics;

operation Oracle_HammingWithPrefix (x : Qubit[], y : Qubit, prefix : Int[]) : Unit {
    // The following line enforces the constraint on the input arrays.
    // You don't need to modify it. Feel free to remove it, this won't cause your code to fail.
    let K = Length(prefix);
    EqualityFactB(1 <= K and K <= Length(x), true, "K should be between 1 and N, inclusive");

    // ...
}

```

### Task 1.8. f(x) = 1 if x has two or three bits (out of three) set to 1, and 0 otherwise (majority function)

**Inputs:**

1. Three qubits in an arbitrary state $|x\rangle$ (input register).
2. A qubit in an arbitrary state $|y\rangle$ (output qubit)

**Goal:** Transform state $|x, y\rangle$ into state $|x, y \oplus f(x)\rangle$ ($\oplus$ is addition modulo 2).

**Hint:** Represent $f(x)$ in terms of AND and $\oplus$ operations.

```qsharp
%kata T18_Oracle_MajorityFunction

open Microsoft.Quantum.Diagnostics;

operation Oracle_MajorityFunction (x : Qubit[], y : Qubit) : Unit {
    // The following line enforces the constraint on the input array.
    // You don't need to modify it. Feel free to remove it, this won't cause your code to fail.
    EqualityFactI(3, Length(x), "x should have exactly 3 qubits");

    // ...
}
```

@[section]({ "id": "deutsch_joza__part2", "title": "Part II. Deutsch-Jozsa Algorithm" })

In this section you will implement the Deutsch-Jozsa algorithm and run it on the oracles you've defined in part I to observe the results.

This algorithm solves the following problem. You are given a quantum oracle which implements a classical function $f(x): \{0, 1\}^N \to \{0, 1\}$. You are guaranteed that the function $f$ is either constant (has the same value for all inputs) or balanced (has value 0 for half of the inputs and 1 for the other half of the inputs). The goal of the algorithm is to figure out whether the function is constant or balanced in just one oracle call.

* You can read more about the Deutsch-Jozsa algorithms and explore its finer points in the [ExploringDeutschJozsaAlgorithm tutorial](https://github.com/microsoft/QuantumKatas/blob/531286588348b322d70d5e053bc60e0be126065e/tutorials/ExploringDeutschJozsaAlgorithm/DeutschJozsaAlgorithmTutorial.ipynb).
* You can read more about the Deutsch-Jozsa algorithm in [Wikipedia](https://en.wikipedia.org/wiki/Deutsch%E2%80%93Jozsa_algorithm).
* [Lecture 5: A simple searching algorithm; the Deutsch-Jozsa algorithm](https://cs.uwaterloo.ca/~watrous/QC-notes/QC-notes.05.pdf).

### Task 2.1. State preparation for Deutsch-Jozsa algorithm

****Inputs:**

1. N qubits in $|0\rangle$ state (query register)
2. A qubit in $|0\rangle$ state (answer register)

**Goal:**

1. Prepare an equal superposition of all basis vectors from $|0...0\rangle$ to $|1...1\rangle$ on query register (that is, state $\frac{1}{\sqrt{2^N}}\big(|0...0\rangle + ... + |1...1\rangle\big)$ ).

2. Prepare $|-\rangle = \frac{1}{\sqrt2} (|0\rangle - |1\rangle)$ state on answer register.

```qsharp
%kata T21_DJ_StatePrep

operation DJ_StatePrep (query : Qubit[], answer : Qubit) : Unit is Adj {
    // ...
}
```

### Task 2.2. Deutsch-Jozsa Algorithm

**Inputs:**

1. The number of $N$ qubits  in the input register for the function f.
2. A quantum operation which implements the oracle $|x, y\rangle \to |x, y \oplus f(x)\rangle$, where x is an $N$-qubit input register, y is a 1-qubit answer register, and f is a Boolean function.

**Output:** `true` if the function f is constant, or `false` if the function f is balanced.

```qsharp
%kata T22_DJ_Algorithm

operation DJ_Algorithm (N : Int, oracle : ((Qubit[], Qubit) => Unit)) : Bool {
    // Create a boolean variable for storing the return value.
    // You'll need to update it later, so it has to be declared as mutable.
    // ...

    // Allocate an array of N qubits for the input register x and one qubit for the answer register y.
    use (x, y) = (Qubit[N], Qubit());
    // Newly allocated qubits start in the |0⟩ state.
    // The first step is to prepare the qubits in the required state before calling the oracle.
    // Each qubit of the input register has to be in the |+⟩ state.
    // ...

    // The answer register has to be in the |-⟩ state.
    // ...

    // Apply the oracle to the input register and the answer register.
    // ...

    // Apply a Hadamard gate to each qubit of the input register again.
    // ...

    // Measure each qubit of the input register in the computational basis using the M operation.
    // If any of the measurement results is One, the function implemented by the oracle is balanced.
    // ...

    // Before releasing the qubits make sure they are all in the |0⟩ state.
    // ...
    
    // Return the answer.
    // ...
}
```

### Task 2.3. Running Deutsch-Jozsa Algorithm

**Goal:** Use your implementation of Deutsch-Jozsa algorithm from task 2.1 to test each of the oracles you've implemented in part I for being constant or balanced.

> This is an open-ended task, and is not covered by a unit test. To run the code, execute the cell with the definition of the `Run_DeutschJozsa_Algorithm` operation first. If it compiled successfully without any errors, you can run the operation by executing the next cell (`%simulate Run_DeutschJozsa_Algorithm).  
> Note that this task relies on your implementations of the previous tasks. If you are getting the "No variable with that name exists." error, you might have to execute previous code cells before retrying this task.

```qsharp
open Microsoft.Quantum.Diagnostics;

operation Run_DeutschJozsa_Algorithm () : String {
    // You can use the Fact function to check that the return value of DJ_Algorithm operation matches the expected value.
    // Uncomment the next line to test the algorithm on the oracle for f(x) = 0.
    // Fact(DJ_Algorithm(4, Oracle_Zero), "f(x) = 0 not identified as constant");
    
    // Run the algorithm for the rest of the oracles
    // ...
    
    // If all tests pass, report success!
    return "Success!";
}
```

```qsharp
%simulate Run_DeutschJozsa_Algorithm
```

@[section]({ "id": "deutsch_joza__part3", "title": "Part III. Bernstein–Vazirani Algorithm" })

In this section you will implement the Bernstein-Vazirani algorithm and run it on the oracles you've defined in part I to observe the results.

This algorithm solves the following problem. You are given a quantum oracle which implements a classical function $f(x): \{0, 1\}^N \to \{0, 1\}$. You are guaranteed that the function $f$
 can be represented as a scalar product. That is, there exists a bit vector $r = (r_0, ..., r_{N-1})$ such that $f(x) = \bigoplus \limits_{i=0}^{N-1} x_i r_i$. . The goal of the algorithm is to reconstruct the bit vector $r$ in just one oracle call.

You can read more about the Bernstein-Vazirani algorithm in [Quantum Algorithm Implementations for Beginners](https://arxiv.org/abs/1804.03719), section III.

### Task 3.1. Bernstein-Vazirani Algorithm

**Inputs:**

1. The number of qubits $N$ in the input register for the function f.
2. A quantum operation which implements the oracle $|x, y\rangle \to |x, y \oplus f(x)\rangle$, where x is an $N$-qubit input register, y is a 1-qubit answer register, and f is a Boolean function.

**Output:** The bit vector $r$ reconstructed from the oracle.

```qsharp
%kata T31_BV_Algorithm

operation BV_Algorithm (N : Int, oracle : ((Qubit[], Qubit) => Unit)) : Int[] {
    // The algorithm is very similar to Deutsch-Jozsa algorithm; try to implement it without hints.
    // ...
}
```

### Task 3.2. Running Bernstein-Vazirani Algorithm

**Goal:** Use your implementation of Bernstein-Vazirani algorithm from task 3.1 to reconstruct the hidden vector $r$ for the oracles you've implemented in part I.

>This is an open-ended task, and is not covered by a unit test. To run the code, execute the cell with the definition of the `Run_BernsteinVazirani_Algorithm` operation first; if it compiled successfully without any errors, you can run the operation by executing the next cell (`%simulate Run_BernsteinVazirani_Algorithm`).  
>Note that this task relies on your implementations of the previous tasks. If you are getting the "No variable with that name exists." error, you might have to execute previous code cells before retrying this task.

**Hint:** Not all oracles from part I can be represented as scalar product functions. The most generic oracle you can use in this task is Oracle_ProductFunction from task 1.5; Oracle_Zero, Oracle_Kth_Qubit and Oracle_OddNumberOfOnes are special cases of this oracle.

```qsharp
open Microsoft.Quantum.Diagnostics;

operation Run_BernsteinVazirani_Algorithm () : String {
    // Now use the library function AllEqualityFactI in Microsoft.Quantum.Diagnostics to verify the results of the algorithm.
    // Uncomment the following lines to test the algorithm on the oracle for f(x) = 0.
    // AllEqualityFactI(BV_Algorithm(3, Oracle_Zero), [0, 0, 0], "Incorrect result for f(x) = 0");
    
    // Run the algorithm on the rest of the oracles
    // ...
    
    // If all tests pass, report success!
    return "Success!";
}
```

```qsharp
%simulate Run_BernsteinVazirani_Algorithm
```

@[section]({ "id": "deutsch_joza__part4", "title": "Part IV. Create your own algorithm!" })

In this section you will create your own algorithm to solve a problem similar to the one described in part III.

The problem is formulated as follows. You are given a quantum oracle which implements a classical function $f(x): \{0, 1\}^N \to \{0, 1\}$. You are guaranteed that there exists a bit vector $r = (r_0, ..., r_{N-1})$ such that the function $f$ can be represented as follows: $f(x) = \bigoplus \limits_{i=0}^{N-1} \left( x_i r_i + (1 - x_i)(1 - r_i) \right)$. You have to reconstruct the bit vector $r$ in just one oracle call.

> Note that you have implemented the oracle for this function in task 1.6.

### Task 4. Noname Algorithm

**Inputs:**

1. The number of qubits $N$ in the input register for the function f.
2. A quantum operation which implements the oracle $|x, y\rangle \to |x, y \oplus f(x)\rangle$, where x is an $N$-qubit input register, y is a 1-qubit answer register, and f is a Boolean function.

**Output:** Any bit vector $r$ that would generate the same oracle as the one you are given.

**Hint:** For each oracle, there are multiple bit vectors that generate it. It is sufficient to find any one of them.

```qsharp
%kata T41_Noname_Algorithm

operation Noname_Algorithm (N : Int, oracle : ((Qubit[], Qubit) => Unit)) : Int[] {
    // The algorithm is very similar to Bernstein-Vazirani algorithm; try to implement it without hints.
    // ...
}
```
