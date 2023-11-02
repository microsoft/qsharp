# Oracles

@[section]({
    "id": "oracles__overview",
    "title": "Overview"
})

Quantum oracles are a key part of many quantum algorithms that rely on quantum implementation of a classical function. The algorithms' discussions often assume that the quantum oracle that implements the function of interest is provided.  This kata dives deeper into the definition of different types of quantum oracles, their properties, and the basic ways to implement the oracles.

**This kata covers the following topics:**

- Quantum oracles and how they relate to classical oracles
- Two types of quantum oracles - phase oracles and marking oracles
- Phase kickback and its uses for oracles implementation
- Implementation and testing of quantum oracles in Q#

**What you should know to start working on this kata:**

- Fundamental quantum concepts
- Multi-qubit gates (especially controlled gates)

@[section]({
    "id": "oracles__classical_oracles",
    "title": "Classical Oracles"
})

In classical computing, we often discuss "black box" versus "white box" testing.  In "white box" testing, the implementation of a function is visible to the tester,  thus they can verify specific runtime or memory complexity expectations for the algorithm.  
However, in "black box" testing, the tester doesn't have access to the details of the function implementation. They only have access to the "black box" that takes an input and produces the corresponding output. This means the tester can only test the functionality and expected behavior of the function, but not the implementation, which is hidden behind abstraction.

Formally, a **classical oracle** is a function that, provided some input, produces a *deterministic* output
(the same input *always* results in the same output).

Some classical problems (typically <a href="https://en.wikipedia.org/wiki/Decision_problem" target="_blank">decision problems</a>) are also expressed in terms of oracles; in this case we do not care about how the function is implemented, but only about the functionality that it provides.  

> Suppose I provided you a function which takes two list parameters as input, where these lists represent the availability of two employees at a company during the week.  The function returns true if there is a day (Monday, Tuesday, Wednesday, Thursday, or Friday) for which they are both free and could schedule a meeting, and false if no such date exists.
>
> This function is an example of a classical oracle.

@[exercise]({
    "id": "oracles__implement_classical_oracles",
    "title": "Implement a Classical Oracle",
    "descriptionPath": "./classical_oracles/index.md",
    "placeholderSourcePath": "./classical_oracles/placeholder.qs",
    "solutionPath": "./classical_oracles/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./classical_oracles/verification.qs"
    ]
})

@[section]({
    "id": "oracles__quantum_oracles",
    "title": "Quantum Oracles"
})

An oracle in the quantum world is a "black box" operation that is used as input to an algorithm (such as Deutsch-Jozsa algorithm or Grover's search algorithm). 
Many quantum algorithms assume an oracle implementation of some classical function as input, but this is a very strong assumption - sometimes implementing the oracle for a function is a lot more complex than the algorithm that will use this oracle!  
In this kata, you will learn the properties of quantum oracles and how to implement them.

A quantum oracle implements a function $f: \\{0,1\\}^n \rightarrow \\{0,1\\}^m$, where the input is $n$-bits of the form $x = (x_{0}, x_{1}, \dots, x_{n-1})$. In most commonly used cases $m=1$, that is, the function can return values $0$ or $1$. In this kata, we will focus on this class of functions.

Quantum oracles operate on qubit arrays (and can take classical parameters as well).  The classical input is encoded into the state of an $n$-qubit register:  
$$|x\rangle = |x_0\rangle \otimes |x_1\rangle \otimes ... \otimes |x_{n-1}\rangle,$$ 
where $|x_i\rangle$ represents the state of the $i$-th qubit.  

Oracles must be unitary transformations, and follow the same rules of linear algebra as other quantum operations.
This allows us to define quantum oracles based on their effect on the basis states - tensor products of single-qubit basis states $|0\rangle$ and $|1\rangle$. 

> For example, an oracle that implements a function that takes 2 bits of input will be defined using its effect on basis states $|00\rangle$, $|01\rangle$, $|10\rangle$, and $|11\rangle$.  

There are two types of quantum oracles: phase oracles and marking oracles.  Let's take a closer look at them.

## Phase Oracles

For a function $f: \\{0,1\\}^n \rightarrow \\{0,1\\}$, the phase oracle $U_{\text{phase}}$ encodes the the values $f(0)$ and $f(1)$ in the relative phases of basis states $\ket{0}$ and $\ket{1}$, respectively.

$$U_{phase} |\vec{x}\rangle = (-1)^{f(x)}|\vec{x}\rangle$$

Thus, the phase oracle $U_{\text{phase}}$ doesn't change the phase of the basis states for which $f(x)=0$, but multiplies the phase of the basis states for which $f(x)=1$ by $-1$.

The effect of such an oracle on any single basis state is not particularly interesting: it just adds a global phase which is not something you can observe. However, if you apply this oracle to a *superposition* of basis states, its effect becomes noticeable. 
Remember that quantum operations are linear: if you define the effect of an operation on the basis states, you'll be able to deduce its effect on superposition states (which are just linear combinations of the basis states) using its linearity. 

A phase oracle doesn't have an "output", unlike the function it implements; the effect of the oracle application is the change in the state of the system.

@[section]({
    "id": "oracles__phase_oracle",
    "title": "Phase Oracle for Alternating Bit Pattern Function"
})

Consider the function $f(x)$ that takes $3$ bits of input and returns $1$ if $x=101$ or $x=010$, and $0$ otherwise.

The phase oracle that implements this function will take an array of 3 qubits as an input, flip the sign of basis states $|101\rangle$ and $|010\rangle$, and leave the rest of the basis states unchanged. Let's see the effect of this oracle on a superposition state.

@[example]({"id": "oracles__phase_oracle_alt_bit", "codePath": "./phase_oracle_alt_bit.qs"})

We introduced the function <a href="https://learn.microsoft.com/en-us/qsharp/api/qsharp/microsoft.quantum.canon.applycontrolledonbitstring" target="_blank">`ApplyControlledOnBitString`</a> provided by the Q# Standard library.
It defines a variant of a gate controlled on a state specified by a bit mask; for example, bit mask `[true, false]` means that the gate should be applied only if the two control qubits are in the $|10\rangle$ state.
 
The sequence of steps that implement this variant are:
1. Apply the $X$ gate to each control qubit that corresponds to a `false` element of the bit mask. After this, if the control qubits started in the $|10\rangle$ state, they'll end up in the $|11\rangle$ state, and if they started in any other state, they'll end up in any state but $|11\rangle$.
2. Apply the regular controlled version of the gate.
3. Apply the $X$ gate to the same qubits to return them to their original state.

Due to this <a href="https://learn.microsoft.com/en-us/azure/quantum/user-guide/language/statements/conjugations" target="_blank">conjugation pattern</a>, the time complexity of this function is $2N$, where N is the number of control qubits.

> Notice that the input state in the demo above is an equal superposition of all basis states. 
After applying the oracle the absolute values of all amplitudes are the same, but the states $|010\rangle$ and $|101\rangle$ had their phase flipped to negative!  
> Recall that these two states are exactly the inputs for which $f(x) = 1$, thus they are exactly the two states we expect to experience a phase flip!

In the next exercise you will implement the classical oracle that you've implemented in the first exercise, this time as a quantum phase oracle $U_{7,\text{phase}}$ that encodes the number 7.

@[exercise]({
    "id": "oracles__phase_oracle_seven",
    "title": "Implement a Phase Oracle",
    "descriptionPath": "./phase_oracle_seven/index.md",
    "placeholderSourcePath": "./phase_oracle_seven/placeholder.qs",
    "solutionPath": "./phase_oracle_seven/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./phase_oracle_seven/verification.qs"
    ]
})

@[section]({
    "id": "oracles__marking_oracles",
    "title": "Marking Oracles"
})

A marking oracle $U_{mark}$ is an oracle that encodes the value of the classical function $f$ it implements in the *amplitude* of the qubit state. When provided an input array of qubits in the basis state $|\vec{x}\rangle$ and an output qubit in the basis state $|y\rangle$, it flips the state of the output qubit if $f(x)=1$. (You can also represent this as addition modulo 2 between $f(x)$ and $y$.)  Hence $U_{mark}$ is an operator that performs the following operation:

$$U_{mark}|\vec{x}\rangle |y\rangle = U_{mark}\big(|\vec{x}\rangle \otimes |y\rangle\big) = |\vec{x}\rangle \otimes |y \oplus f(x)\rangle$$

Again, since all quantum operations are linear, you can figure out the effect of this operation on superposition state knowing its effect on the basis states using its linearity. 

A marking oracle has distinct "input" and "output" qubits, but in general the effect of the oracle application is the change in the state of the whole system rather than of the "output" qubits only. We will look at this closer in a moment.

## Marking Oracle for Alternating Bit Pattern Function

Consider the function $f(x)$ that takes $3$ bits of input and returns $1$ if $x=101$ or $x=010$, and $0$ otherwise (it is the same function we've seen in the demo "Phase oracle for alternating bit pattern function").

The marking oracle that implements this function will take an array of 3 qubits as an "input" register and an "output" qubit, and will flip the state of the output qubit if the input qubit was in basis state $|101\rangle$ or $|010\rangle$, and do nothing otherwise. Let's see the effect of this oracle on a superposition state.

@[example]({"id": "oracles__marking_oracle_alt_bit", "codePath": "./marking_oracle_alt_bit.qs"})

> Let's compare the initial state to the final state from the above demo. 
In the initial state we had a tensor product of an equal superposition of all 3-qubit basis states and the state $|0\rangle$.  In the final state, this is no longer the case. 
The basis states $|010\rangle \otimes |0\rangle$ and $|101\rangle \otimes |0\rangle$ no longer have non-zero amplitudes, and instead $|010\rangle \otimes |1\rangle$ and $|101\rangle \otimes |1\rangle$ have non-zero amplitudes.
>
> This is exactly the result that we expect.  Recall our function $f(x)$: $f(x)=1$ if and only if $x=010$ or $x=101$.  The first three qubits (variable `x`) represent the input state $|x\rangle$, and the last qubit (variable `y`) represents the output state $|y\rangle$.  Thus when we have the two basis states, $|x\rangle=|010\rangle$ or $|x\rangle=|101\rangle$, we will flip the state of the qubit $|y\rangle$, causing these two initial states to be tensored with $|1\rangle$ in the final state where originally they were tensored with $|0\rangle$.
>
> Since the rest of the basis states correspond to $f(x) = 0$, all other basis states in the initial superposition remain unchanged.

Now you will implement the same function you've seen in the first two exercises as a marking oracle $U_{7,mark}$.

@[exercise]({
    "id": "oracles__marking_oracle_seven",
    "title": "Implement a Marking Oracle",
    "descriptionPath": "./marking_oracle_seven/index.md",
    "placeholderSourcePath": "./marking_oracle_seven/placeholder.qs",
    "solutionPath": "./marking_oracle_seven/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./marking_oracle_seven/verification.qs"
    ]
})

@[section]({
    "id": "oracles__phase_kickback",
    "title": "Phase Kickback"
})

Previously we considered applying marking oracles when the register $|x\rangle$ was in a basis state or a superposition state, and the target qubit $|y\rangle$ in a basis state.  How might the effect of applying marking oracles change if the target is also in a superposition state?  In this case we will observe **phase kickback** - the relative phase from the target qubit affecting ("kicked back" into) the state of the input qubits.

In order to observe phase kickback, we use the target qubit $|y\rangle=|-\rangle$.

> This is the standard choice for two reasons. 
> First, for phase kickback to occur, the target qubit must have a difference in relative phase between the basis states $|0\rangle$ and $|1\rangle$. 
> Second, the target qubit must be in an equal superposition, otherwise it will become entangled with the input register.

Let's see the results of applying a marking oracle $U_{mark}$ which implements the function $f(x)$ to the input register $|x\rangle$ and the target qubit in state $|-\rangle$:
* If the input register $|x\rangle$ is in a basis state:

$$U_{mark} |x\rangle |-\rangle = \frac1{\sqrt2} \big(U_{mark}|x\rangle|0\rangle - U_{mark}|x\rangle |1\rangle\big)$$

$$= \frac1{\sqrt2} \big(|x\rangle|0\oplus f(x)\rangle - |x\rangle |1\oplus f(x)\rangle\big)$$

$$= 
\begin{cases} 
\frac1{\sqrt2} \big(|x\rangle|0\rangle - |x\rangle |1\rangle\big) = |x\rangle|-\rangle \text{ if } f(x) = 0 \\\ 
\frac1{\sqrt2} \big(|x\rangle|1\rangle - |x\rangle |0\rangle\big) = -|x\rangle|-\rangle \text{ if } f(x) = 1
\end{cases}
$$

$$= (-1)^{f(x)}|x\rangle |-\rangle$$


* If the input register is in a superposition state, say $|x\rangle = \frac1{\sqrt2} \big(|b_1\rangle + |b_2\rangle\big)$, where $|b_1\rangle$ and $|b_2\rangle$ are basis states:

$$U_{mark} |x\rangle |-\rangle = U_{mark} \frac{1}{\sqrt{2}} \big(|b_1\rangle + |b_2\rangle\big) |-\rangle$$

$$= \frac{1}{\sqrt{2}} \big( U_{mark}|b_1\rangle|-\rangle + U_{mark}|b_2\rangle|-\rangle\big)$$

$$= \frac{1}{\sqrt{2}} \big( (-1)^{f(b_1)}|b_1\rangle + (-1)^{f(b_2)}|b_2\rangle\big) |-\rangle$$

We see that in both cases applying $U_{mark}$ does not change the state of the target qubit, but it does change the state of the input register. 
Thus we can drop the target qubit without any repercussions after the application of the oracle. 
Notice that the input register is now in the following state:
$$|\psi\rangle = \frac{1}{\sqrt{2}} \big( (-1)^{f(b_1)}|b_1\rangle + (-1)^{f(b_2)}|b_2\rangle\big),$$

which looks exactly as if we applied a phase oracle to $|x\rangle$ instead of applying a marking oracle to $|x\rangle|-\rangle$!  This is a very important application of phase kickback: it allows to convert a marking oracle into a phase oracle - which you will implement in the next task.

> Another important application of this effect is **phase estimation** algorithm, which allows to estimate an eigenvalue of an eigenvector.

Consider the following example using the $U_{7,mark}$ oracle. Let's begin with $|x\rangle$ as an equal superposition of the $6$ and $7$ basis states and $|y\rangle=|-\rangle$, the overall state is:

$$|\eta\rangle = \Big[\frac{1}{\sqrt{2}}\big(|110\rangle + |111\rangle\big)\Big] \otimes \frac{1}{\sqrt{2}}\big(|0\rangle - |1\rangle\big)$$

$$= \frac{1}{2} \big(|110\rangle|0\rangle + |111\rangle|0\rangle - |110\rangle|1\rangle - |111\rangle|1\rangle\big)$$

How does $U_{7,mark}$ act on this state?

$$U_{7,mark}|\eta\rangle = U_{7,mark} \frac{1}{2} \big(|110\rangle|0\rangle + |111\rangle|0\rangle - |110\rangle|1\rangle - |111\rangle|1\rangle \big)$$

$$= \frac{1}{2} \big( U_{7,mark}|110\rangle|0\rangle + U_{7,mark}|111\rangle|0\rangle - U_{7,mark}|110\rangle|1\rangle - U_{7,mark}|111\rangle|1\rangle \big)$$

$$= \frac{1}{2} \big(|110\rangle|0\rangle + |111\rangle|1\rangle - |110\rangle|1\rangle - |111\rangle|0\rangle \big) := |\xi\rangle$$

Now we would like to observe how our input state $|\eta\rangle$ was modified by the oracle.  Let's simplify the resulting state $|\xi\rangle$:

$$|\xi\rangle = \frac{1}{2} \big(|110\rangle|0\rangle + |111\rangle|1\rangle - |110\rangle|1\rangle - |111\rangle|0\rangle\big)$$

$$= \frac{1}{2} \big(|110\rangle|0\rangle - |110\rangle|1\rangle - |111\rangle|0\rangle + |111\rangle|1\rangle \big)$$

$$= \frac{1}{2} \Big[|110\rangle \otimes \big(|0\rangle - |1\rangle \big) + |111\rangle \otimes \big(|1\rangle - |0\rangle\big)\Big]$$

$$= \Big[\frac{1}{\sqrt{2}} \big( |110\rangle - |111\rangle \big) \Big] \otimes \Big[ \frac{1}{\sqrt{2}} \big( |0\rangle - |1\rangle \big) \Big]$$

$$= \Big[\frac{1}{\sqrt{2}} \big( |110\rangle - |111\rangle \big) \Big] \otimes |-\rangle$$

Finally, let's compare $|\eta\rangle$ and $|\xi\rangle$; below are the final equations repeated for your convenience:
$$|\eta\rangle = \Big[\frac{1}{\sqrt{2}}\big(|110\rangle + |111\rangle\big)\Big] \otimes |-\rangle$$
$$|\xi\rangle = \Big[\frac{1}{\sqrt{2}}\big(|110\rangle - |111\rangle\big)\Big] \otimes |-\rangle$$

We can see that these two equations are identical, except for the $-1$ phase that appeared on the $|111\rangle$ basis state (representing $7$).  This is a specific example of the phase kickback effect, as the phase from $|-\rangle$ has been *kicked back* into $|x\rangle$.

## 🔎 Analyze

**Distinguishing states**

How could we distinguish the states $|\eta\rangle = |11+\rangle |-\rangle$ and $|\xi\rangle = |11-\rangle |-\rangle$?  Take a moment to think.

<details>
<summary><b>Solution</b></summary>
Recall that we can only observe alterations to out input state by performing a measurement.
If we apply Hadamard gate to the third qubit, we will be able to distinguish between the input state and the output state. 
    $$(I\otimes I \otimes H)|11+\rangle = |110\rangle \\ (I\otimes I \otimes H)|11-\rangle = |111\rangle$$ 
Now if we were to measure the third qubit, we'll be able to distinguish the starting state and the state after phase kickback occurred.
</details>

@[exercise]({
    "id": "oracles__marking_oracle_as_phase",
    "title": "Apply the Marking Oracle as a Phase Oracle",
    "descriptionPath": "./marking_oracle_as_phase/index.md",
    "placeholderSourcePath": "./marking_oracle_as_phase/placeholder.qs",
    "solutionPath": "./marking_oracle_as_phase/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./marking_oracle_as_phase/verification.qs"
    ]
})

@[section]({
    "id": "oracles__conversion",
    "title": "Oracle Conversion"
})

In this demo we will use a reference implementation of `ApplyMarkingOracleAsPhaseOracle` operation to convert marking oracle `IsSeven_MarkingOracle` to a phase oracle. Then we will compare this converted oracle to the reference implementation of the phase oracle `IsSeven_PhaseOracle`. You already implemented these oracles in the previous tasks.

@[example]({"id": "oracles__oracle_converter_demo", "codePath": "./oracle_converter_demo.qs"})

> Notice from the above demo that your phase oracle $U_{7,phase}$ behaves the same as the converted version of your marking oracle $U_{7,mark}$, both of which induce a phase flip on the basis state $|111\rangle$!

This way to convert a marking oracle to a phase oracle is useful because many quantum algorithms, such as Grover's search algorithm, rely on a phase oracle, but it is often easier to implement the function as a marking oracle. 
This converter provides a way to implement the function of interest as a marking oracle and then convert it into a phase oracle, which could then be leveraged in a quantum algorithm.

@[section]({
    "id": "oracles__implementing_quantum_oracles",
    "title": "Implementing Quantum Oracles"
})

In this section you will implement a few more complicated quantum oracles. 

> Notice that the operation declarations below require adjoint and controlled variants of the oracle to be automatically generated. This is common practice that makes testing and reusing the code easier. Typically Q# compiler will easily generate these variants, as long as you don't use mutable variables or operations that don't support these functors.

@[exercise]({
    "id": "oracles__or_oracle",
    "title": "Implement the OR Oracle",
    "descriptionPath": "./or_oracle/index.md",
    "placeholderSourcePath": "./or_oracle/placeholder.qs",
    "solutionPath": "./or_oracle/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./or_oracle/verification.qs"
    ]
})

@[exercise]({
    "id": "oracles__kth_bit_oracle",
    "title": "Implement the K-th Bit Oracle",
    "descriptionPath": "./kth_bit_oracle/index.md",
    "placeholderSourcePath": "./kth_bit_oracle/placeholder.qs",
    "solutionPath": "./kth_bit_oracle/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./kth_bit_oracle/verification.qs"
    ]
})

@[exercise]({
    "id": "oracles__or_but_kth_oracle",
    "title": "Implement the OR Oracle of All Bits Except the K-th",
    "descriptionPath": "./or_but_kth_oracle/index.md",
    "placeholderSourcePath": "./or_but_kth_oracle/placeholder.qs",
    "solutionPath": "./or_but_kth_oracle/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./or_but_kth_oracle/verification.qs"
    ]
})

@[exercise]({
    "id": "oracles__bit_pattern_oracle",
    "title": "Implement the Arbitrary Bit Pattern Oracle",
    "descriptionPath": "./bit_pattern_oracle/index.md",
    "placeholderSourcePath": "./bit_pattern_oracle/placeholder.qs",
    "solutionPath": "./bit_pattern_oracle/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./bit_pattern_oracle/verification.qs"
    ]
})

@[exercise]({
    "id": "oracles__bit_pattern_challenge",
    "title": "Implement the Arbitrary Bit Pattern Oracle (Challenge Version)",
    "descriptionPath": "./bit_pattern_challenge/index.md",
    "placeholderSourcePath": "./bit_pattern_challenge/placeholder.qs",
    "solutionPath": "./bit_pattern_challenge/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./bit_pattern_challenge/verification.qs"
    ]
})

@[exercise]({
    "id": "oracles__meeting_oracle",
    "title": "Implement the Meeting Oracle",
    "descriptionPath": "./meeting_oracle/index.md",
    "placeholderSourcePath": "./meeting_oracle/placeholder.qs",
    "solutionPath": "./meeting_oracle/solution.md",
    "codePaths": [
        "../KatasLibrary.qs",
        "./common.qs",
        "./meeting_oracle/verification.qs"
    ]
})

@[section]({
    "id": "oracles__testing_implementation",
    "title": "Testing an Oracle Implementation"
})

In this demo we show how you could test an oracle that you've implemented for your own problem. 
For all of the previous oracles that you've implemented, we've been testing your oracle against a reference solution for that task. 
However, if you're designing an oracle for a new problem, you do not have a reference solution for it - if you did, there would be no point for you to program the oracle in the first place!

A good way to test a quantum oracle of interest is to write a classical oracle that performs the same computation classically, and then compare the effect of your quantum oracle on the basis states with the output of the classical oracle for every input (or a lot of the inputs if you are constrained by runtime) to ensure that they match.

Here we will compare the reference implementation of `Meeting_Oracle` to the classical code implementing the same function.

@[example]({"id": "oracles__test_meeting_oracle", "codePath": "./test_meeting_oracle.qs"})

@[section]({
    "id": "oracles__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you have learned to build quantum oracles. Here are a few key concepts to keep in mind:
* A quantum oracle is an "opaque box" operation that is used as input to another algorithm.
* Phase oracles encode the information in the relative phase of basis states. If $f(x)=0$, the oracle doesn't change the basis state $\ket{x}$, and if $f(x)=1$ it multiplies the phase of the basis state $\ket{x}$ by $-1$.
* Marking oracles use an extra qubit $\ket{y}$ and encode the information in the state of that qubit. If $f(x)=0$, it doen't change the state of the qubit $\ket{y}$ for the basis state $\ket{x}$, and if $f(x)=1$ it flips the state of the qubit $\ket{y}$ for the basis state $\ket{x}$.

**Next Steps**

We hope you enjoyed this kata! If you're looking to learn more about quantum oracles and Q#, here are some suggestions:
* To learn about the Grover's algorithm, you can check <a href="https://learn.microsoft.com/en-us/training/modules/solve-graph-coloring-problems-grovers-search/" target="_blank">Microsoft Learn module "Solve graph coloring problems by using Grover's search"</a>.
* To learn more about the Q# libraries, you can check the <a href="https://learn.microsoft.com/en-us/azure/quantum/user-guide/libraries/?tabs=tabid-clivscode" target="_blank">The Q# user guide</a>.
