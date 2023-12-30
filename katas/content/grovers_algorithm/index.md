# Grover's Algorithm

@[section]({ "id": "grovers_algorithm__overview", "title": "Overview" })

Grover's search algorithm is one of the fundamental quantum computing algorithms. it solves the problem of finding an input to a black box (oracle) that produces a particular output.

This kata consists of a series of exercises designed to get you familiar with Grover's algorithm. It covers the following topics:

* writing oracles for Grover's search,
* performing steps of the algorithm, and
* putting it all together: Grover's search algorithm.

Within each section, tasks are given in approximate order of increasing difficulty. The most difficult tasks are marked with an asterisk.

Each task is wrapped in one operation preceded by the description of the task. Your goal is to fill in the blanks (marked with the `// ...` comments) with some Q# code that solves the task. To verify your answer, run the cell with Ctrl+Enter (⌘+Enter on macOS).

*For more information about Grover's algorithm, see the following:*

* The [Oracles tutorial](../oracles/index.md) is an introduction into quantum oracles.
* This [Microsoft Learn module](https://docs.microsoft.com/learn/modules/solve-graph-coloring-problems-grovers-search/) offers a different, visual explanation of Grover's algorithm.
* The tasks follow the explanation from Quantum Computation and Quantum Information by Nielsen and Chuang. In the  10th anniversary edition, this is section 6.1.2 on pages 248-251.
* A different explanation of Grover's algorithm can be found in this Wikipedia article.
* [An Introduction to Quantum Algorithms](https://strubell.github.io/doc/quantum_tutorial.pdf) by Emma Strubell, pages 20-24.
* Lecture [4: Grover's Algorithm](https://www.cs.cmu.edu/~odonnell/quantum15/lecture04.pdf) by John Wright.
* Quantum Computation, Lectures [12](https://cs.uwaterloo.ca/~watrous/QC-notes/QC-notes.12.pdf) and [13](https://cs.uwaterloo.ca/~watrous/QC-notes/QC-notes.13.pdf) by John Watrous.
* [Animation of Grover's Quantum Search Algorithm](http://davidbkemp.github.io/animated-qubits/grover.html) by David  Kemp has an animated demonstration of Grover's algorithm for a simple case.

@[section]({ "id": "grovers_algorithm__part1", "title": "Part I. Oracles for Grover's Search" })

## Task 1.1. The $|11...1\rangle$ Oracle

**Inputs:**

1. N qubits in an arbitrary state $|x\rangle$ (input/query register)

2. A qubit in an arbitrary state $|y\rangle$ (target qubit)

**Goal:**

Flip the state of the target qubit (i.e., apply an X gate to it)
if the query register is in the $|11...1\rangle$ state,
and leave it unchanged if the query register is in any other state.
Leave the query register in the same state it started in.

**Examples:**

* If the query register is in state $|00...0\rangle$, leave the target qubit unchanged.

* If the query register is in state $|10...0\rangle$, leave the target qubit unchanged.

* If the query register is in state $|11...1\rangle$, flip the target qubit.

* If the query register is in state $\\frac{1}{\\sqrt{2}} \\big(|00...0\rangle + |11...1\rangle \\big)$, and the target is in state $|0\rangle$,
the joint state of the query register and the target qubit should be $\\frac{1}{\\sqrt{2}} \\big(|00...00\rangle + |11...11\rangle \\big)$."

```qsharp
%kata T11_Oracle_AllOnes

operation Oracle_AllOnes (queryRegister : Qubit[], target : Qubit) : Unit is Adj {
  // ...

  }   
```
  
## Task 1.2. The $|1010...\rangle$ Oracle

**Inputs:**

1. N qubits in an arbitrary state $|x\rangle$ (input/query register)

2. A qubit in an arbitrary state $|y\rangle$ (target qubit)

**Goal:**

Flip the state of the target qubit if the query register is in the $|1010...\rangle$ state;
that is, the state with alternating 1 and 0 values, with any number of qubits in the register.
Leave the state of the target qubit unchanged if the query register is in any other state.
Leave the query register in the same state it started in.

**Examples:**

* If the register is in state $|0000000\rangle$, leave the target qubit unchanged.

* If the register is in state $|10101\rangle$, flip the target qubit."

```qsharp
%kata T12_Oracle_AlternatingBits

operation Oracle_AlternatingBits (queryRegister : Qubit[], target : Qubit) : Unit is Adj {
  // ...
} 
```

## Task 1.3. Arbitrary Bit Pattern Oracle

**Inputs:**

1. N qubits in an arbitrary state $|x\rangle$ (input/query register)

2. A qubit in an arbitrary state $|y\rangle$ (target qubit)

3. A bit pattern of length N represented as `Bool[]`

**Goal:**

Flip the state of the target qubit if the query register is in the state described by the given bit pattern
(`true` represents qubit state One, and `false` represents Zero).
Leave the state of the target qubit unchanged if the query register is in any other state.
Leave the query register in the same state it started in.

**Example:**

If the bit pattern is `[true, false]`, you need to flip the target qubit if and only if the qubits are in the $|10\rangle$ state."  

```qsharp
%kata T13_Oracle_ArbitraryPattern 

operation Oracle_ArbitraryPattern (queryRegister : Qubit[], target : Qubit, pattern : Bool[]) : Unit is Adj {
  // ...
} 
```

## Task 1.4. Oracle Converter

**Input:**

A marking oracle: an oracle that takes a register and a target qubit and
flips the target qubit if the register satisfies a certain condition.

**Output:**

A phase-flipping oracle: an oracle that takes a register and
flips the phase of the register if it satisfies this condition.

  Grover's algorithm relies on the search condition implemented as a phase-flipping oracle,
    but it is often easier to write a marking oracle for a given condition. This transformation
    allows to convert one type of oracle into the other. The transformation is described in the
    [Wikipedia article on Grover's algorithm](https://en.wikipedia.org/wiki/Grover%27s_algorithm), in the section \"Description of ${U_\omega}$\".

```qsharp
kata T14_OracleConverter

function OracleConverter (markingOracle : ((Qubit[], Qubit) => Unit is Adj)) : (Qubit[] => Unit is Adj) {
  // ...
}
```

**Hint**
  Remember that you can define auxiliary operations. To do that, you'll need to create an extra code cell for each new operation and execute it before returning to this cell.

@[section]({ "id": "grovers_algorithm__part2", "title": "Part II. The Grover Iteration" })

## Task 2.1. The Hadamard Transform

**Input:** A register of N qubits in an arbitrary state

**Goal:** Apply the Hadamard transform to each of the qubits in the register.

> If the register started in the $|0...0\rangle$ state, this operation
will prepare an equal superposition of all $2^{N}$ basis states."

```qsharp
%kata T21_HadamardTransform 

operation HadamardTransform (register : Qubit[]) : Unit is Adj {
  // ...
}
```

## Task 2.2. Conditional Phase Flip

**Input:**  A register of N qubits in an arbitrary state.

**Goal:**  Flip the sign of the state of the register if it is not in the $|0...0\rangle$ state.

**Examples:**

* If the register is in state $|0...0\rangle$, leave it unchanged.

* If the register is in any other basis state, multiply its phase by -1.

This operation implements operator $2|0...0\rangle\langle0...0| - I$ $ = \left(\begin{matrix}1&0&...&0\\\\0&-1&...&0\\\\\vdots&\vdots&\ddots&\vdots\\\\0&0&...&-1\end{matrix}\right) $

```qsharp
%kata T22_ConditionalPhaseFlip 

operation ConditionalPhaseFlip (register : Qubit[]) : Unit is Adj {   // ... }
```

**Hint #1**
  Note that quantum states are defined up to a global phase.
  Thus the state obtained as a result of this operation is equivalent to the state obtained by flipping the sign of only the $|0...0\rangle$ basis state (those states differ by a global phase $-1$).
> $$-\big(2|0...0\rangle\langle0...0| - I\big) = I - 2|0...0\rangle\langle0...0| = \left(\begin{matrix}-1&0&...&0\\\\0&1&...&0\\\\\vdots&\vdots&\ddots&\vdots\\\\0&0&...&1\end{matrix}\right) $$
>It doesn't matter for Grover's search algorithm itself, since the global phase is not observable, but can have side effects when used as part of other algorithms.
> See the extended discussion in this [Quantum Computing SE question](https://quantumcomputing.stackexchange.com/questions/5973/counting-in-q-number-of-solutions/6446#6446)

**Hint #2**
  Consider the Controlled Z gate, applied with most of the qubits as control and the last qubit as target:
 >$\text{Controlled Z}(|s_0 s_1 \ldots s_{n-2}\rangle, |s_{n-1}\rangle)$ leaves all basis states except $|1...11\rangle$ unchanged, and adds a $-1$ phase to that state: $|1...11\rangle \rightarrow -|1...11\rangle$ (remember that $Z|0\rangle = |0\rangle$ and $Z|1\rangle = -|1\rangle$).
>
>You need to modify it to add the $-1$ phase to only the $|0...00\rangle$ state instead.
>
>Alternatively, you can use the same trick as in the oracle converter task.

**Hint #3**
You can use the [R gate](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.intrinsic.r) to correct the global phase.

## Task 2.3. The Grover Iteration

**Inputs:**

1. N qubits in an arbitrary state $|x\rangle$ (input/query register)

2. A phase-flipping oracle that takes an N-qubit register and flips the phase of the state if the register is in the desired state.

**Goal:**  Perform one Grover iteration.

```qsharp
%kata T23_GroverIteration 

operation GroverIteration (register : Qubit[], oracle : (Qubit[] => Unit is Adj)) : Unit is Adj {  // ... }
```

**Hint:**
A Grover iteration consists of 4 steps:
>
> * Apply the Oracle
> * Apply the Hadamard transform
> * Perform a conditional phase shift
> * Apply the Hadamard transform again

@[section]({ "id": "grovers_algorithm__part3", "title": "Part III. Putting It All Together: Grover's Search Algorithm" })

## Task 3.1. Grover's Search

**Inputs:**

1. N qubits in the $|0...0\rangle$ state.

2. A marking oracle.

3. The number of Grover iterations to perform.

      >**Note:** The number of iterations is passed as a parameter because it is defined by the nature of the problem and is easier to configure/calculate outside the search algorithm itself (for example, in the classical driver).

**Goal:** Use Grover's algorithm to leave the register in the state that is marked by the oracle as the answer (with high probability).

```qsharp
 %kata T31_GroversSearch operation GroversSearch (register : Qubit[], oracle : ((Qubit[], Qubit) => Unit is Adj), iterations : Int) : Unit {   // ... }
```

## Task 3.2. Using Grover's Search

**Goal:**   Use your implementation of Grover's Algorithm from Task 3.1 and the oracles from part 1
to find the marked elements of the search space. This task is not covered by a test and allows you to experiment with running the algorithm.

 This is an open-ended task, and is not covered by a unit test. To run the code, execute the cell with the definition of the `Run_GroversSearch_Algorithm` operation first. If it compiles successfully without any errors, you can run the operation by executing the next cell (`%simulate Run_GroversSearch_Algorithm`).
Note that this task relies on your implementations of the previous tasks. If you are getting the \"No variable with that name exists.\" error, you might have to execute previous code cells before retrying this task.  

```qsharp
 operation Run_GroversSearch_Algorithm () : Unit {
  // ...
} 
```

```qsharp
%simulate Run_GroversSearch_Algorithm 
```

**Hint #1**
To check whether the algorithm found the correct answer (i.e., an answer marked as 1 by the oracle),
you can apply the oracle once more to the register after you've measured it and an ancilla qubit,
which will calculate the function of the answer found by the algorithm.

**Hint #2**
Experiment with the number of iterations to see how it affects
the probability of the algorithm finding the correct answer.

**Hint #3**
You can use the Message function to output the results.
