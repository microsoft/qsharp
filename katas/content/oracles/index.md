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
    "path": "./classical_oracles/"
})

@[section]({
    "id": "oracles__quantum_oracles",
    "title": "Quantum Oracles"
})

An oracle in the quantum world is a "black box" operation that is used as input to an algorithm (such as Deutsch-Jozsa algorithm or Grover's search algorithm).
Many quantum algorithms assume an oracle implementation of some classical function as input, but this is a very strong assumption - sometimes implementing the oracle for a function is a lot more complex than the algorithm that will use this oracle!  
In this kata, you will learn the properties of quantum oracles and how to implement them.

A quantum oracle implements a function $f: \{0,1\}^n \rightarrow \{0,1\}^m$, where the input is $n$-bits of the form $x = (x_{0}, x_{1}, \dots, x_{n-1})$. In most commonly used cases $m=1$, that is, the function can return values $0$ or $1$. In this kata, we will focus on this class of functions.

Quantum oracles operate on qubit arrays (and can take classical parameters as well).  The classical input is encoded into the state of an $n$-qubit register:  
$$\ket{\vec{x}} = \ket{x_0} \otimes \ket{x_1} \otimes ... \otimes \ket{x_{n-1}},$$
where $\ket{x_i}$ represents the state of the $i$-th qubit.  

Oracles must be unitary transformations, and follow the same rules of linear algebra as other quantum operations.
This allows us to define quantum oracles based on their effect on the basis states - tensor products of single-qubit basis states $\ket{0}$ and $\ket{1}$.

> For example, an oracle that implements a function that takes two bits of input will be defined using its effect on basis states $\ket{00}$, $\ket{01}$, $\ket{10}$, and $\ket{11}$.  

There are two types of quantum oracles: phase oracles and marking oracles.  Let's take a closer look at them.

@[section]({
    "id": "oracles__phase_oracles",
    "title": "Phase Oracles"
})

For a function $f: \{0,1\}^n \rightarrow \{0,1\}$, the phase oracle $U_{\text{phase}}$ encodes the values of the function $f$ in the *relative phases* of basis states. When provided an input basis state $\ket{\vec{x}}$, it flips the sign of that state if $f(x)=1$:

$$U_{phase} \ket{\vec{x}} = (-1)^{f(x)}\ket{\vec{x}}$$

Thus, the phase oracle $U_{\text{phase}}$ doesn't change the phase of the basis states for which $f(x)=0$, but multiplies the phase of the basis states for which $f(x)=1$ by $-1$.

The effect of such an oracle on any single basis state is not particularly interesting: it just adds a global phase which is not something you can observe. However, if you apply this oracle to a *superposition* of basis states, its effect becomes noticeable.
Remember that quantum operations are linear: if you define the effect of an operation on the basis states, you'll be able to deduce its effect on superposition states (which are just linear combinations of the basis states) using its linearity.

A phase oracle doesn't have an "output", unlike the function it implements; the effect of the oracle application is the change in the state of the system.

@[section]({
    "id": "oracles__phase_oracle_alternating_bits",
    "title": "Phase Oracle for Alternating Bit Pattern Function"
})

Consider the function $f(x)$ that takes three bits of input and returns $1$ if $x=101$ or $x=010$, and $0$ otherwise.

The phase oracle that implements this function will take an array of three qubits as an input, flip the sign of basis states $\ket{101}$ and $\ket{010}$, and leave the rest of the basis states unchanged. Let's see the effect of this oracle on a superposition state.

@[example]({"id": "oracles__phase_oracle_alt_bit", "codePath": "./examples/PhaseOracleAltBit.qs"})

We introduced the function `ApplyControlledOnBitString` provided by the Q# standard library when we discussed controlled gates.
It defines a variant of a gate controlled on a state specified by a bit mask; for example, bit mask `[true, false]` means that the gate should be applied only if the two control qubits are in the $\ket{10}$ state.

The sequence of steps that implement this variant are:

1. Apply the $X$ gate to each control qubit that corresponds to a `false` element of the bit mask. After this, if the control qubits started in the $\ket{10}$ state, they'll end up in the $\ket{11}$ state, and if they started in any other state, they'll end up in any state but $\ket{11}$.
2. Apply the regular controlled version of the gate.
3. Apply the $X$ gate to the same qubits to return them to their original state.

> Notice that the input state in the demo above is an equal superposition of all basis states.
After applying the oracle the absolute values of all amplitudes are the same, but the states $\ket{010}$ and $\ket{101}$ had their phase flipped to negative.
> Recall that these two states are exactly the inputs for which $f(x) = 1$, thus they are exactly the two states we expect to experience a phase flip!

In the next exercise you will implement the classical oracle that you've implemented in the first exercise, this time as a quantum phase oracle $U_{7,\text{phase}}$ that encodes the number 7.

@[exercise]({
    "id": "oracles__phase_oracle_seven",
    "title": "Implement a Phase Oracle",
    "path": "./phase_oracle_seven/"
})

@[section]({
    "id": "oracles__marking_oracles",
    "title": "Marking Oracles"
})

A marking oracle $U_{mark}$ is an oracle that encodes the value of the classical function $f$ it implements in the *amplitude* of the qubit state. When provided an input array of qubits in the basis state $\ket{\vec{x}}$ and an output qubit in the basis state $\ket{y}$, it flips the state of the output qubit if $f(x)=1$. (You can also represent this as addition modulo 2 between $f(x)$ and $y$.)  Hence $U_{mark}$ is an operator that performs the following operation:

$$U_{mark}\ket{\vec{x}} \ket{y} = U_{mark}\big(\ket{\vec{x}} \otimes \ket{y}\big) = \ket{\vec{x}} \otimes \ket{y \oplus f(x)}$$

Again, since all quantum operations are linear, you can figure out the effect of this operation on superposition state knowing its effect on the basis states using its linearity.

A marking oracle has distinct "input" and "output" qubits, but in general the effect of the oracle application is the change in the state of the whole system rather than of the "output" qubits only. We will look at this closer in a moment.

@[section]({
    "id": "oracles__marking_oracle_alternating_bits",
    "title": "Marking Oracle for Alternating Bit Pattern Function"
})

Consider the function $f(x)$ that takes three bits of input and returns $1$ if $x=101$ or $x=010$, and $0$ otherwise (it is the same function we've seen in the lesson "Phase Oracle for Alternating Bit Pattern Function").

The marking oracle that implements this function will take an array of three qubits as an "input" register and an "output" qubit, and will flip the state of the output qubit if the input qubit was in basis state $\ket{101}$ or $\ket{010}$, and do nothing otherwise. Let's see the effect of this oracle on a superposition state.

@[example]({"id": "oracles__marking_oracle_alt_bit", "codePath": "./examples/MarkingOracleAltBit.qs"})

> Let's compare the initial state to the final state from the above demo.
In the initial state we had a tensor product of an equal superposition of all three-qubit basis states and the state $\ket{0}$.  In the final state, this is no longer the case.
The basis states $\ket{010} \otimes \ket{0}$ and $\ket{101} \otimes \ket{0}$ no longer have non-zero amplitudes, and instead $\ket{010} \otimes \ket{1}$ and $\ket{101} \otimes \ket{1}$ have non-zero amplitudes.
>
> This is exactly the result that we expect.  Recall our function $f(x)$: $f(x)=1$ if and only if $x=010$ or $x=101$.  The first three qubits (variable `x`) represent the input state $\ket{x}$, and the last qubit (variable `y`) represents the output state $\ket{y}$.  Thus when we have the two basis states, $\ket{x}=\ket{010}$ or $\ket{x}=\ket{101}$, we will flip the state of the qubit $\ket{y}$, causing these two initial states to be tensored with $\ket{1}$ in the final state where originally they were tensored with $\ket{0}$.
>
> Since the rest of the basis states correspond to $f(x) = 0$, all other basis states in the initial superposition remain unchanged.

Now you will implement the same function you've seen in the first two exercises as a marking oracle $U_{7,mark}$.

@[exercise]({
    "id": "oracles__marking_oracle_seven",
    "title": "Implement a Marking Oracle",
    "path": "./marking_oracle_seven/"
})

@[section]({
    "id": "oracles__phase_kickback",
    "title": "Phase Kickback"
})

Previously we considered applying marking oracles when the register $\ket{x}$ was in a basis state or a superposition state, and the target qubit $\ket{y}$ was in a basis state.  How might the effect of applying marking oracles change if the target is also in a superposition state?  In this case we might observe **phase kickback** - the relative phase from the target qubit affecting ("kicked back" into) the state of the input qubits.

In order to observe phase kickback, we use the target qubit $\ket{y}=\ket{-}$.

> This is the standard choice for two reasons.
> First, for phase kickback to occur, the target qubit must have a difference in relative phase between the basis states $\ket{0}$ and $\ket{1}$.
> Second, the absolute values of the amplitudes of the two basis states of the target qubit must be equal, otherwise the target will become entangled with the input register.

Let's see the results of applying a marking oracle $U_{mark}$ which implements the function $f(x)$ to the input register $\ket{x}$ and the target qubit in state $\ket{-}$:

- If the input register $\ket{x}$ is in a basis state:

$$U_{mark} \ket{x} \ket{-} = \frac1{\sqrt2} \big(U_{mark}\ket{x}\ket{0} - U_{mark}\ket{x} \ket{1}\big) =$$

$$= \frac1{\sqrt2} \big(\ket{x}\ket{0\oplus f(x)} - \ket{x} \ket{1\oplus f(x)}\big) =$$

$$=\begin{cases}
\frac1{\sqrt2} \big(\ket{x}\ket{0} - \ket{x} \ket{1}\big) = \ket{x}\ket{-} \text{ if } f(x) = 0 \\
\frac1{\sqrt2} \big(\ket{x}\ket{1} - \ket{x} \ket{0}\big) = -\ket{x}\ket{-} \text{ if } f(x) = 1
\end{cases}=$$

$$= (-1)^{f(x)}\ket{x} \ket{-}$$

- If the input register is in a superposition state, say $\ket{x} = \frac1{\sqrt2} \big(\ket{b_1} + \ket{b_2}\big)$, where $\ket{b_1}$ and $\ket{b_2}$ are basis states:

$$U_{mark} \ket{x} \ket{-} = U_{mark} \frac{1}{\sqrt{2}} \big(\ket{b_1} + \ket{b_2}\big) \ket{-} =$$

$$= \frac{1}{\sqrt{2}} \big( U_{mark}\ket{b_1}\ket{-} + U_{mark}\ket{b_2}\ket{-}\big) =$$

$$= \frac{1}{\sqrt{2}} \big( (-1)^{f(b_1)}\ket{b_1} + (-1)^{f(b_2)}\ket{b_2}\big) \ket{-}$$

We see that in both cases applying $U_{mark}$ does not change the state of the target qubit, but it does change the state of the input register.
Thus we can drop the target qubit without any repercussions after the application of the oracle.
Notice that the input register is now in the following state:
$$\ket{\psi} = \frac{1}{\sqrt{2}} \big( (-1)^{f(b_1)}\ket{b_1} + (-1)^{f(b_2)}\ket{b_2}\big),$$

which looks exactly as if we applied a phase oracle to $\ket{x}$ instead of applying a marking oracle to $\ket{x}\ket{-}$!  This is a very important application of phase kickback: it allows to convert a marking oracle into a phase oracle - which you will implement in the next task.

> Another important application of this effect is **phase estimation** algorithm, which allows to estimate an eigenvalue of an eigenvector.

Consider the following example using the $U_{7,mark}$ oracle. Let's begin with $\ket{x}$ as an equal superposition of the $\ket{110}$ and $\ket{111}$ basis states and $\ket{y}=\ket{-}$, the overall state is:

$$\ket{\eta} = \Big[\frac{1}{\sqrt{2}}\big(\ket{110} + \ket{111}\big)\Big] \otimes \frac{1}{\sqrt{2}}\big(\ket{0} - \ket{1}\big) =$$

$$= \frac{1}{2} \big(\ket{110}\ket{0} + \ket{111}\ket{0} - \ket{110}\ket{1} - \ket{111}\ket{1}\big)$$

How does $U_{7,mark}$ act on this state?

$$U_{7,mark}\ket{\eta} = U_{7,mark} \frac{1}{2} \big(\ket{110}\ket{0} + \ket{111}\ket{0} - \ket{110}\ket{1} - \ket{111}\ket{1} \big) =$$

$$= \frac{1}{2} \big( U_{7,mark}\ket{110}\ket{0} + U_{7,mark}\ket{111}\ket{0} - U_{7,mark}\ket{110}\ket{1} - U_{7,mark}\ket{111}\ket{1} \big) =$$

$$= \frac{1}{2} \big(\ket{110}\ket{0} + \ket{111}\ket{1} - \ket{110}\ket{1} - \ket{111}\ket{0} \big) := \ket{\xi}$$

Now we would like to observe how our input state $\ket{\eta}$ was modified by the oracle.  Let's simplify the resulting state $\ket{\xi}$:

$$\ket{\xi} = \frac{1}{2} \big(\ket{110}\ket{0} + \ket{111}\ket{1} - \ket{110}\ket{1} - \ket{111}\ket{0}\big) =$$

$$= \frac{1}{2} \big(\ket{110}\ket{0} - \ket{110}\ket{1} - \ket{111}\ket{0} + \ket{111}\ket{1} \big) =$$

$$= \frac{1}{2} \Big[\ket{110} \otimes \big(\ket{0} - \ket{1} \big) + \ket{111} \otimes \big(\ket{1} - \ket{0}\big)\Big] =$$

$$= \Big[\frac{1}{\sqrt{2}} \big( \ket{110} - \ket{111} \big) \Big] \otimes \Big[ \frac{1}{\sqrt{2}} \big( \ket{0} - \ket{1} \big) \Big] =$$

$$= \Big[\frac{1}{\sqrt{2}} \big( \ket{110} - \ket{111} \big) \Big] \otimes \ket{-}$$

Finally, let's compare $\ket{\eta}$ and $\ket{\xi}$; below are the final equations repeated for your convenience:
$$\ket{\eta} = \Big[\frac{1}{\sqrt{2}}\big(\ket{110} + \ket{111}\big)\Big] \otimes \ket{-}$$
$$\ket{\xi} = \Big[\frac{1}{\sqrt{2}}\big(\ket{110} - \ket{111}\big)\Big] \otimes \ket{-}$$

We can see that these two equations are identical, except for the $-1$ phase that appeared on the $\ket{111}$ basis state - our marked state.  This is a specific example of the phase kickback effect, as the phase from $\ket{-}$ has been *kicked back* into $\ket{x}$.


@[exercise]({
    "id": "oracles__marking_oracle_as_phase",
    "title": "Apply the Marking Oracle as a Phase Oracle",
    "path": "./marking_oracle_as_phase/"
})

@[section]({
    "id": "oracles__conversion",
    "title": "Converting Marking Oracles to Phase Oracles"
})

In this demo we will use a reference implementation of `ApplyMarkingOracleAsPhaseOracle` operation to convert marking oracle `IsSeven_MarkingOracle` to a phase oracle. Then we will compare this converted oracle to the reference implementation of the phase oracle `IsSeven_PhaseOracle`. You already implemented these oracles in the previous tasks.

@[example]({"id": "oracles__oracle_converter_demo", "codePath": "./examples/OracleConverterDemo.qs"})

> Notice from the above demo that your phase oracle $U_{7,phase}$ behaves the same as the converted version of your marking oracle $U_{7,mark}$, both of which induce a phase flip on the basis state $\ket{111}$!

This way to convert a marking oracle to a phase oracle is useful because many quantum algorithms, such as Grover's search algorithm, rely on a phase oracle, but it is often easier to implement the function as a marking oracle.
This converter provides a way to implement the function of interest as a marking oracle and then convert it into a phase oracle, which could then be leveraged in a quantum algorithm.

@[section]({
    "id": "oracles__implementing_quantum_oracles",
    "title": "Implementing Quantum Oracles"
})

In this section you will implement a few more complicated quantum oracles.
Some of them - both phase and marking - can take extra "classical" parameters.
A useful tool for implementing quantum oracles is allocating auxiliary qubits to assist in a computation. 
You will practice that in some of the exercises below.

> Notice that the operation declarations below require adjoint and controlled variants of the oracle to be automatically generated. This is common practice that makes testing and reusing the code easier. Typically Q# compiler will easily generate these variants, as long as you don't use mutable variables or operations that don't support these functors.

@[exercise]({
    "id": "oracles__or_oracle",
    "title": "Implement the OR Oracle",
    "path": "./or_oracle/"
})

@[exercise]({
    "id": "oracles__kth_bit_oracle",
    "title": "Implement the K-th Bit Oracle",
    "path": "./kth_bit_oracle/"
})

@[exercise]({
    "id": "oracles__or_but_kth_oracle",
    "title": "Implement the OR Oracle of All Bits Except the K-th",
    "path": "./or_but_kth_oracle/"
})

@[exercise]({
    "id": "oracles__bit_pattern_oracle",
    "title": "Implement the Arbitrary Bit Pattern Oracle",
    "path": "./bit_pattern_oracle/"
})

@[exercise]({
    "id": "oracles__bit_pattern_challenge",
    "title": "Implement the Arbitrary Bit Pattern Oracle (Challenge Version)",
    "path": "./bit_pattern_challenge/"
})

@[exercise]({
    "id": "oracles__meeting_oracle",
    "title": "Implement the Meeting Oracle",
    "path": "./meeting_oracle/"
})

@[section]({
    "id": "oracles__testing_implementation",
    "title": "Testing an Oracle Implementation"
})

In this demo we show how you could test an oracle that you've implemented for your own problem.
For all of the previous oracles that you've implemented, we've been testing your oracle against a reference solution for that task.
However, if you're designing an oracle for a new problem, you do not have a reference solution for it - if you did, there would be no point for you to implement the oracle in the first place!

A good way to test a quantum oracle of interest is to write a classical oracle that performs the same computation classically, and then compare the effect of your quantum oracle on the basis states with the output of the classical oracle for every input (or a certain percentage of the inputs if you are constrained by runtime) to ensure that they match.

Here we will compare the reference implementation of `Meeting_Oracle` to the classical code implementing the same function.

@[example]({"id": "oracles__test_meeting_oracle", "codePath": "./examples/TestMeetingOracle.qs"})

@[section]({
    "id": "oracles__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you have learned to build quantum oracles. Here are a few key concepts to keep in mind:

- A quantum oracle is an "opaque box" operation that implements a classical computation. 
- Quantum oracles are used to convert classical problems into inputs to quantum algorithms, such as Grover's search algorithm.
- Phase oracles encode the information in the relative phase of basis states. If $f(x)=0$, the oracle doesn't change the basis state $\ket{x}$, and if $f(x)=1$, it multiplies the phase of the basis state $\ket{x}$ by $-1$.
- Marking oracles use an extra qubit $\ket{y}$ and encode the information in the state of that qubit. If $f(x)=0$, the oracle doesn't change the state of the qubit $\ket{y}$ for the basis state $\ket{x}$, and if $f(x)=1$, it flips the state of the qubit $\ket{y}$ for the basis state $\ket{x}$.
