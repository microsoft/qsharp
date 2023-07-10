# Oracles Tutorial

Quantum oracles are a key part of many quantum algorithms that rely on quantum implementation of a classical function. The algorithms' discussions often assume that the quantum oracle that implements the function of interest is provided.  This tutorial dives deeper into the definition of different types of quantum oracles, their properties, and the basic ways to implement the oracles.

This tutorial will:
* introduce you to quantum oracles and how they relate to classical oracles,
* explain two types of quantum oracles - phase oracles and marking oracles,
* introduce phase kickback and its uses for oracles implementation,
* teach you to implement quantum oracles in Q# and to test your implementations.

Before diving into the material, we recommend you to make sure you're comfortable with the fundamental quantum concepts, in particular [basic quantum computing gates](../MultiQubitGates/MultiQubitGates.ipynb) (especially controlled gates).

Let's get started!

# Part I. Introduction to Quantum Oracles

## Classical Oracles
In classical computing, we often discuss "black box" versus "white box" testing.  In "white box" testing, the implementation of a function is visible to the tester,  thus they can verify specific runtime or memory complexity expectations for the algorithm.  
However, in "black box" testing the tester doesn't have access to the details of the function implementation, but only to the "box" that takes an input and produces the corresponding output. This means the tester can only test the functionality and expected behavior of the function, but not the implementation has been abstracted away.

Formally, a **classical oracle** is a function that, provided some input, produces a *deterministic* output
(the same input *always* results in the same output).

Some classical problems (typically [decision problems](https://en.wikipedia.org/wiki/Decision_problem)) are also expressed in terms of oracles; in this case we do not care about how the function is implemented, but only about the functionality that it provides.  

> Suppose I provided you a function which takes two list parameters as input, where these lists represent the availability of two employees at a company during the week.  The function returns true if there is a day (Monday, Tuesday, Wednesday, Thursday, or Friday) for which they are both free and could schedule a meeting, and false if no such date exists.
>
> This function is an example of a classical oracle.

### <span style="color:blue">Task 1.1</span>: Implement a classical oracle
**Input:** 
  A bit vector of length 3 represented as a `Bool[]` - a binary representation of a number.

**Output:**
  Return `true` if the input array represents the number $7$, and `false` otherwise.

**Examples:**

* If the input array is `[true, true, true]`, return `true`.
* If the input array is `[true, true, false]`, return `false`.

@[exercise]({
"id": "classical_oracles",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./Common.qs"
],
"verificationSourcePath": "./classical_oracles/verification.qs",
"placeholderSourcePath": "./classical_oracles/placeholder.qs",
"solutionSourcePath": "./classical_oracles/solution.qs",
"solutionDescriptionPath": "./solution.md"
})

## Quantum Oracles

An oracle in the quantum world is a "black box" operation that is used as input to an algorithm (such as Deutsch-Jozsa algorithm or Grover's search algorithm which you'll learn later). 
Many quantum algorithms assume an oracle implementation of some classical function as input, but this is a very strong assumption - sometimes implementing the oracle for a function is a lot more complex than the algorithm that will use this oracle!  
In this tutorial you will learn the properties of quantum oracles and how to implement them.

A quantum oracle implements a function $f: \{0,1\}^n \rightarrow \{0,1\}^m$, where $x$ is an $n$-bit input state
of the form $x = (x_{0}, x_{1}, \dots, x_{n-1})$. In most commonly used cases $m=1$, i.e., the function can return values $0$ or $1$; in this tutorial we will focus on this class of functions.

Quantum oracles operate on qubit arrays (and can take classical parameters as well).  The classical input is encoded into the state of an $n$-qubit register:  
$$|x\rangle = |x_0\rangle \otimes |x_1\rangle \otimes ... \otimes |x_{n-1}\rangle,$$ 
where $|x_i\rangle$ represents the state of the $i$-th qubit.  

Oracles must be unitary transformations, and follow the same rules of linear algebra as other quantum operations. (See the [linear algebra tutorial](../LinearAlgebra/LinearAlgebra.ipynb) if you need a refresher.)
This allows us to define quantum oracles based on their effect on the basis states - tensor products of single-qubit basis states $|0\rangle$ and $|1\rangle$. 

> For example, an oracle that implements a function that takes 2 bits of input will be defined using its effect on basis states $|00\rangle$, $|01\rangle$, $|10\rangle$, and $|11\rangle$.  

There are two types of quantum oracles: phase oracles and marking oracles.  Let's take a closer look at them.

### Phase Oracles
A phase oracle $U_{phase}$ is an oracle that encodes the value of the classical function $f$ it implements in the *phase* of the qubit state. When provided an input basis state $|\vec{x}\rangle$, it flips the sign of that state if $f(x)=1$:

$$U_{phase} |\vec{x}\rangle = (-1)^{f(x)}|\vec{x}\rangle$$

The effect of such an oracle on any single basis state is not particularly interesting: it just adds a global phase which is not something you can observe. However, if you apply this oracle to a *superposition* of basis states, its effect becomes noticeable. 
Remember that quantum operations are linear: if you define the effect of an operation on the basis states, you'll be able to deduce its effect on superposition states (which are just linear combinations of the basis states) using its linearity. 

A phase oracle doesn't have an "output", unlike the function it implements; the effect of the oracle application is the change in the state of the system.

### <span style="color:blue">Demo 1.1</span>: Phase oracle for alternating bit pattern function

Consider the function $f(x)$ that takes $3$ bits of input and returns $1$ if $x=101$ or $x=010$, and $0$ otherwise.

The phase oracle that implements this function will take an array of 3 qubits as an input, flip the sign of basis states $|101\rangle$ and $|010\rangle$, and leave the rest of the basis states unchanged. Let's see the effect of this oracle on a superposition state.

@[example]({"id": "phase_oracle_alt_bit", "codePath": "./phase_oracle_alt_bit.qs"})

We introduced the function [ControlledOnBitString](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.canon.controlledonbitstring) provided by the Q# Standard library.
It defines a variant of a gate controlled on a state specified by a bit mask; for example, bit mask `[true, false]` means that the gate should be applied only if the two control qubits are in the $|10\rangle$ state.
 
The sequence of steps that implement this variant are:
1. Apply the $X$ gate to each control qubit that corresponds to a `false` element of the bit mask. After this, if the control qubits started in the $|10\rangle$ state, they'll end up in the $|11\rangle$ state, and if they started in any other state, they'll end up in any state but $|11\rangle$.
2. Apply the regular controlled version of the gate.
3. Apply the $X$ gate to the same qubits to return them to their original state.

Due to this [conjugation pattern](https://learn.microsoft.com/en-us/azure/quantum/user-guide/language/statements/conjugations), the time complexity of this function is 2 * N, where N is the number of control qubits. To learn its internal implementation (and the very similar [ControlledOnInt](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.canon.controlledonint)), please refer to the [Q# source code](https://github.com/microsoft/QuantumLibraries/blob/c0b851735542117cf6d73f8946ab6eef8c84384d/Standard/src/Canon/Utils/ControlledOnBitString.qs#L107).

> Notice that the input state in the demo above is an equal superposition of all basis states. 
After applying the oracle the absolute values of all amplitudes are the same, but the states $|010\rangle$ and $|101\rangle$ had their phase flipped to negative!  
> Recall that these two states are exactly the inputs for which $f(x) = 1$, thus they are exactly the two states we expect to experience a phase flip!

Now you will implement the classical oracle that you've implemented in task 1.1 as a quantum phase oracle $U_{7,phase}$.

### <span style="color:blue">Task 1.2</span>: Implement a phase oracle

**Input:**
  3 qubits in an arbitrary state $|x\rangle$ (input/query register).

**Goal:**

Flip the sign of the input state $|x\rangle$ if the input register is in
the state $|111\rangle$ (encoding the integer $7$), and leave the input register unchanged otherwise.  
Don't allocate extra qubits to perform this operation.

**Examples:**

* If the query register is in the state $|111\rangle$, flip its sign.
* If the query register is in the state $|010\rangle$ or $|101\rangle$, do nothing.

<details>
  <summary><b>Need a hint? Click here</b></summary>
  To solve this problem, you need to find a gate that will only flip the sign of the $|111\rangle$ basis state.  Which single-qubit gate flips the sign of the basis state $|1\rangle$ but not $|0\rangle$? How can you modify this gate to solve this problem?
</details>

@[exercise]({
"id": "phase_oracle_seven",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./Common.qs"
],
"verificationSourcePath": "./phase_oracle_seven/verification.qs",
"placeholderSourcePath": "./phase_oracle_seven/placeholder.qs",
"solutionSourcePath": "./phase_oracle_seven/solution.qs",
"solutionDescriptionPath": "./solution.md"
})

<details>
    <summary><b>For a closer look at the mathematical properties of this oracle, click here</b></summary>

Consider how the oracle from task 1.2 acts on two basis states:
$$U_{7,phase} |111\rangle = -|111\rangle$$
$$U_{7,phase} |110\rangle = |110\rangle$$

You can see that $U_{7,phase}$ does not change the input if it's a basis state (other than adding a global phase), and $U_{7,phase}$ does not change the norm of the state ($U_{7,phase}$ is a unitary operator).  

However, if we applied this oracle to a superposition state instead, what will that look like?

Suppose that $|\beta\rangle$ is an equal superposition of the $|6\rangle$ and $|7\rangle$ states (encoded in big endian, with most significant bit first): 
$$|\beta\rangle = \frac{1}{\sqrt{2}} \big(|110\rangle + |111\rangle\big) = |11\rangle \otimes \frac{1}{\sqrt{2}} \big(|0\rangle + |1\rangle\big) = |11\rangle \otimes |+\rangle = |11+\rangle$$

Let's consider how our operator $U_{7,phase}$ acts on this state:
$$U_{7,phase} |\beta\rangle = U_{7,phase} \Big[\frac{1}{\sqrt{2}} \big(|110\rangle + |111\rangle\big)\Big] = $$
$$= \frac{1}{\sqrt{2}} \big(U_{7,phase} |110\rangle + U_{7,phase} |111\rangle\big) = \frac{1}{\sqrt{2}} \big(|110\rangle - |111\rangle\big) := |\gamma\rangle$$

Was our input state modified during this operation? Let's simplify $|\gamma\rangle$:
$$|\gamma\rangle = \frac{1}{\sqrt{2}} \big(|110\rangle - |111\rangle\big) = |11\rangle \otimes \frac{1}{\sqrt{2}} \big(|0\rangle - |1\rangle\big) = $$
$$= |11\rangle \otimes |-\rangle = |11-\rangle \neq |\beta\rangle$$

Here we see that the oracle modifies the input, if the input state was a *superposition* of the basis states, as a phase oracle will only modify the sign of the basis states.  Thus when a superposition state is provided as input to an oracle, the input state can be modified via the application of the quantum oracle.

> It is also worth noting that while the oracle modified the input when provided a superposition state, it did *not* modify the norm of that state.  As an exercise, you can verify this yourself by taking the norm of $|\beta\rangle$ and $|\gamma\rangle$, which both will result in a value of $1$.
>
> As another exercise, consider how you could distinguish between the input and output state programmatically?  Is there an operation that you could apply to the initial state $|\beta\rangle$ and the final state $|\gamma\rangle$ to show that the two states are not equivalent through measurement?  As a hint, think about how you could convert the superposition states $|\beta\rangle$ and $|\gamma\rangle$ into the basis states.

</details>

### Marking Oracles

A marking oracle $U_{mark}$ is an oracle that encodes the value of the classical function $f$ it implements in the *amplitude* of the qubit state. When provided an input array of qubits in the basis state $|\vec{x}\rangle$ and an output qubit in the basis state $|y\rangle$, it flips the state of the output qubit if $f(x)=1$. (You can also represent this as addition modulo 2 between $f(x)$ and $y$.)  Hence $U_{mark}$ is an operator that performs the following operation:

$$U_{mark}|\vec{x}\rangle |y\rangle = U_{mark}\big(|\vec{x}\rangle \otimes |y\rangle\big) = |\vec{x}\rangle \otimes |y \oplus f(x)\rangle$$

Again, since all quantum operations are linear, you can figure out the effect of this operation on superposition state knowing its effect on the basis states using its linearity. 

A marking oracle has distinct "input" and "output" qubits, but in general the effect of the oracle application is the change in the state of the whole system rather than of the "output" qubits only. We will look at this closer in a moment.

### <span style="color:blue">Demo 1.2</span>: Marking oracle  for alternating bit pattern function

Consider the function $f(x)$ that takes $3$ bits of input and returns $1$ if $x=101$ or $x=010$, and $0$ otherwise (it is the same function we've seen in demo 1.1).

The marking oracle that implements this function will take an array of 3 qubits as an "input" register and an "output" qubit, and will flip the state of the output qubit if the input qubit was in basis state $|101\rangle$ or $|010\rangle$, and do nothing otherwise. Let's see the effect of this oracle on a superposition state.

@[example]({"id": "phase_oracle_alt_bit", "codePath": "./phase_oracle_alt_bit.qs"})

> Let's compare the initial state to the final state from the above demo. 
In the initial state we had a tensor product of an equal superposition of all 3-qubit basis states and the state $|0\rangle$.  In the final state, this is no longer the case. 
The basis states $|010\rangle \otimes |0\rangle$ and $|101\rangle \otimes |0\rangle$ no longer have non-zero amplitudes, and instead $|010\rangle \otimes |1\rangle$ and $|101\rangle \otimes |1\rangle$ has non-zero amplitudes.
>
> This is exactly the result that we expect.  Recall our function $f(x)$: $f(x)=1$ if and only if $x=010$ or $x=101$.  The first three qubits (variable `x`) represent the input state $|x\rangle$, and the last qubit (variable `y`) represents the output state $|y\rangle$.  Thus when we have the two basis states, $|x\rangle=|010\rangle$ or $|x\rangle=|101\rangle$, we will flip the state of the qubit $|y\rangle$, causing these two initial states to be tensored with $|1\rangle$ in the final state where originally they were tensored with $|0\rangle$.
>
> Since the rest of the basis states correspond to $f(x) = 0$, all other basis states in the initial superposition remain unchanged.

Now you will implement the same function you've seen in the first two tasks as a marking oracle $U_{7,mark}$.

### <span style="color:blue">Task 1.3</span>: Implement a marking oracle

**Inputs:**

  1. 3 qubits in an arbitrary state $|x\rangle$ (input/query register)
    
  2. A qubit in an arbitrary state $|y\rangle$ (target qubit)

**Goal:**

Flip the state of $|y\rangle$ if the input register is in the 
state $|111\rangle$, and leave the state $|y\rangle$ unchanged otherwise.

**Examples:**

* If the query register is in the state $|111\rangle$, flip the state of the target qubit $|y\rangle$.
* If the query register is in the state $|010\rangle$ or $|101\rangle$, do nothing.

@[exercise]({
"id": "marking_oracle_seven",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./Common.qs"
],
"verificationSourcePath": "./marking_oracle_seven/verification.qs",
"placeholderSourcePath": "./marking_oracle_seven/placeholder.qs",
"solutionSourcePath": "./marking_oracle_seven/solution.qs",
"solutionDescriptionPath": "./solution.md"
})

<details>
    <summary><b>For a closer look at the mathematical properties of this oracle, click here</b></summary>

Consider how the oracle from task 1.3 acts on two input basis states and two "output" basis states:

$$U_{7,mark} |111\rangle |0\rangle = |111\rangle |0 \oplus f(111)\rangle = |111\rangle |0 \oplus 1\rangle = |111\rangle |1\rangle$$
$$U_{7,mark} |111\rangle |1\rangle = |111\rangle |1 \oplus f(111)\rangle = |111\rangle |1 \oplus 1\rangle = |111\rangle |0\rangle$$

$$U_{7,mark} |110\rangle |0\rangle = |110\rangle |0 \oplus f(110)\rangle = |110\rangle |0 \oplus 0\rangle = |110\rangle |0\rangle$$
$$U_{7,mark} |110\rangle |1\rangle = |110\rangle |1 \oplus f(110)\rangle = |110\rangle |1 \oplus 0\rangle = |110\rangle |1\rangle$$

You can see that the state of the input qubit array is unchanged, and the state of the output qubit changes if $f(x) = 1$ and is unchanged if $f(x) = 0$ - this matches the definition of a marking oracle precisely.

Now let's again apply this oracle to a superposition state $|\alpha\rangle$ such that $|x\rangle$ is a superposition of the $|6\rangle$ and $|7\rangle$ basis states and $|y\rangle = |0\rangle$:
$$|\alpha\rangle = \frac{1}{\sqrt{2}}\big(|110\rangle + |111\rangle\big)|0\rangle = 
|11\rangle \otimes \frac{1}{\sqrt{2}} \big(|0\rangle + |1\rangle\big) \otimes |0\rangle = |11+\rangle |0\rangle$$

Let's consider how our operator $U_{7,mark}$ acts on this state.

> Recall that oracles are linear operators, thus they can be applied to each term individually.

$$U_{7,mark} |\alpha\rangle = \frac{1}{\sqrt{2}} \big(U_{7,mark}|110\rangle |0\rangle + U_{7,mark}|111\rangle |0\rangle\big) =$$
$$= \frac{1}{\sqrt{2}} \big(|110\rangle |0\rangle + |111\rangle |1\rangle\big) := |\epsilon\rangle$$

Was our input state modified during this operation?  Let's simplify the resulting state $|\epsilon\rangle$:
$$|\epsilon\rangle = \frac{1}{\sqrt{2}} \big(|110\rangle |0\rangle + |111\rangle |1\rangle\big) = |11\rangle \otimes \frac{1}{\sqrt{2}} \big(|0\rangle |0\rangle + |1\rangle |1\rangle\big) = $$
$$= |11\rangle \otimes \frac{1}{\sqrt{2}} \big(|00\rangle + |11\rangle\big) = |11\rangle \otimes |\Phi^+\rangle = |11\Phi^+\rangle$$

We have entangled the states of qubits $|x\rangle$ and $|y\rangle$!  This is a common occurrence for marking oracles when the input is a superposition of basis states: after applying the oracle, the input $|x\rangle$ will often become entangled with $|y\rangle$. Thus, while applying the marking oracle to a basis state will leave the input array unchanged, applying the marking oracle to a superposition state will change the state of both the input array and the output qubit.

>As an exercise, what entangled state would we get in the previous example if $|y\rangle = |1\rangle$ instead of $|y\rangle = |0\rangle$?
>
> <br/>
> <details>
>   <summary><b>Click here for the answer!</b></summary>
> $$U_{7,mark} |11+\rangle |1\rangle = |11\rangle \otimes \frac1{\sqrt2}\big(|01\rangle + |10\rangle\big) = |11\rangle |\Psi^+\rangle$$
> </details>

</details>

# Part II: Phase Kickback

Previously we considered applying marking oracles when the register $|x\rangle$ was in a basis state or a superposition state, and the target qubit $|y\rangle$ in a basis state.  How might the effect of applying marking oracles change if the target is also in a superposition state?  In this case we will observe **phase kickback** - the relative phase from the target qubit affecting ("kicked back" into) the state of the input qubits.

In order to observe phase kickback, we use the target qubit $|y\rangle=|-\rangle$.  

> This is the standard choice for two reasons. 
> First, for phase kickback to occur, the target qubit must have a difference in relative phase between the basis states $|0\rangle$ and $|1\rangle$. 
> Second, the target qubit must be in an equal superposition, otherwise it will become entangled with the input register.

Let's see the results of applying a marking oracle $U_{mark}$ which implements the function $f(x)$ to the input register $|x\rangle$ and the target qubit in state $|-\rangle$:
* If the input register $|x\rangle$ is in a basis state:
$$U_{mark} |x\rangle |-\rangle = \frac1{\sqrt2} \big(U_{mark}|x\rangle|0\rangle - U_{mark}|x\rangle |1\rangle\big) 
= \frac1{\sqrt2} \big(|x\rangle|0\oplus f(x)\rangle - |x\rangle |1\oplus f(x)\rangle\big) = \\ 
= \begin{cases} 
    \frac1{\sqrt2} \big(|x\rangle|0\rangle - |x\rangle |1\rangle\big) = |x\rangle|-\rangle \text{ if } f(x) = 0 \\ 
    \frac1{\sqrt2} \big(|x\rangle|1\rangle - |x\rangle |0\rangle\big) = -|x\rangle|-\rangle \text{ if } f(x) = 1
  \end{cases} 
= (-1)^{f(x)}|x\rangle |-\rangle$$


* If the input register is in a superposition state, say $|x\rangle = \frac1{\sqrt2} \big(|b_1\rangle + |b_2\rangle\big)$, where $|b_1\rangle$ and $|b_2\rangle$ are basis states:
$$U_{mark} |x\rangle |-\rangle = U_{mark} \frac{1}{\sqrt{2}} \big(|b_1\rangle + |b_2\rangle\big) |-\rangle = 
 \frac{1}{\sqrt{2}} \big( U_{mark}|b_1\rangle|-\rangle + U_{mark}|b_2\rangle|-\rangle\big) = \\
= \frac{1}{\sqrt{2}} \big( (-1)^{f(b_1)}|b_1\rangle + (-1)^{f(b_2)}|b_2\rangle\big) |-\rangle$$

We see that in both cases applying $U_{mark}$ does not change the state of the target qubit, but it does change the state of the input register. 
Thus we can drop the target qubit without any repercussions after the application of the oracle. 
Notice that the input register is now in the following state:
$$|\psi\rangle = \frac{1}{\sqrt{2}} \big( (-1)^{f(b_1)}|b_1\rangle + (-1)^{f(b_2)}|b_2\rangle\big),$$

which looks exactly as if we applied a phase oracle to $|x\rangle$ instead of applying a marking oracle to $|x\rangle|-\rangle$!  This is a very important application of phase kickback: it allows to convert a marking oracle into a phase oracle - which you will implement in the next task.

> Another important application of this effect is **phase estimation** algorithm, which allows to estimate an eigenvalue of an eigenvector. You can learn more about this important algorithm in the [PhaseEstimation kata](../../PhaseEstimation/PhaseEstimation.ipynb).

<details>
    <summary><b>If you would like to see an example of phase kickback using oracles we've previously seen in this tutorial, click here</b></summary>

Consider the following example using the $U_{7,mark}$ oracle. Let's begin with $|x\rangle$ as an equal superposition of the $6$ and $7$ basis states and $|y\rangle=|-\rangle$, the overall state is:
$$|\eta\rangle = \Big[\frac{1}{\sqrt{2}}\big(|110\rangle + |111\rangle\big)\Big] \otimes \frac{1}{\sqrt{2}}\big(|0\rangle - |1\rangle\big) = $$
$$ = \frac{1}{2} \big(|110\rangle|0\rangle + |111\rangle|0\rangle - |110\rangle|1\rangle - |111\rangle|1\rangle\big)$$

How does $U_{7,mark}$ act on this state?
$$U_{7,mark}|\eta\rangle = U_{7,mark} \frac{1}{2} \big(|110\rangle|0\rangle + |111\rangle|0\rangle - |110\rangle|1\rangle - |111\rangle|1\rangle \big) = $$
$$= \frac{1}{2} \big( U_{7,mark}|110\rangle|0\rangle + U_{7,mark}|111\rangle|0\rangle - U_{7,mark}|110\rangle|1\rangle - U_{7,mark}|111\rangle|1\rangle \big) = $$
$$= \frac{1}{2} \big(|110\rangle|0\rangle + |111\rangle|1\rangle - |110\rangle|1\rangle - |111\rangle|0\rangle \big) := |\xi\rangle$$
    
Now we would like to observe how our input state $|\eta\rangle$ was modified by the oracle.  Let's simplify the resulting state $|\xi\rangle$:
$$|\xi\rangle = \frac{1}{2} \big(|110\rangle|0\rangle + |111\rangle|1\rangle - |110\rangle|1\rangle - |111\rangle|0\rangle\big)  = $$
$$= \frac{1}{2} \big(|110\rangle|0\rangle - |110\rangle|1\rangle - |111\rangle|0\rangle + |111\rangle|1\rangle \big) = $$
$$= \frac{1}{2} \Big[|110\rangle \otimes \big(|0\rangle - |1\rangle \big) + |111\rangle \otimes \big(|1\rangle - |0\rangle\big)\Big] = $$
$$ = \Big[\frac{1}{\sqrt{2}} \big( |110\rangle - |111\rangle \big) \Big] \otimes \Big[ \frac{1}{\sqrt{2}} \big( |0\rangle - |1\rangle \big) \Big] = $$
$$= \Big[\frac{1}{\sqrt{2}} \big( |110\rangle - |111\rangle \big) \Big] \otimes |-\rangle$$

Finally, let's compare $|\eta\rangle$ and $|\xi\rangle$; below are the final equations repeated for your convenience:
$$|\eta\rangle = \Big[\frac{1}{\sqrt{2}}\big(|110\rangle + |111\rangle\big)\Big] \otimes |-\rangle$$
$$|\xi\rangle = \Big[\frac{1}{\sqrt{2}}\big(|110\rangle - |111\rangle\big)\Big] \otimes |-\rangle$$

We can see that these two equations are identical, except for the $-1$ phase that appeared on the $|111\rangle$ basis state (representing $7$).  This is a specific example of the phase kickback effect, as the phase from $|-\rangle$ has been *kicked back* into $|x\rangle$.
    
<br/>
<details>
  <summary><b>How could we distinguish the states $|\eta\rangle = |11+\rangle |-\rangle$ and $|\xi\rangle = |11-\rangle |-\rangle$?  Take a moment to think, then click here to see if you were correct</b></summary>
Recall that we can only observe alterations to out input state by performing a measurement.
If we apply Hadamard gate to the third qubit, we will be able to distinguish between the input state and the output state. 
    $$(I\otimes I \otimes H)|11+\rangle = |110\rangle \\ (I\otimes I \otimes H)|11-\rangle = |111\rangle$$ 
Now if we were to measure the third qubit, we'll be able to distinguish the starting state and the state after phase kickback occurred.
</details>
    
</details>

### <span style="color:blue">Task 2.1</span>: Apply the marking oracle as a phase oracle
**Inputs:**

  1. A marking oracle implementing an unknown $N$-bit function $f(x)$.
  2. $N$ qubits in an arbitrary state (input/query register).
  
**Goal:**

Flip the phase of each basis state $|x\rangle$ for which $f(x) = 1$. You can only access $f(x)$ via the marking oracle you are given.

<br/>
<details>
  <summary><b>Need a hint? Click here</b></summary>
    Recall that you can allocate extra qubits to assist in this operation.  Is there a state that you could prepare with an auxiliary qubit which would help you to convert the marking oracle to a phase oracle?
</details>

@[exercise]({
"id": "marking_oracle_as_phase",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./Common.qs"
],
"verificationSourcePath": "./marking_oracle_as_phase/verification.qs",
"placeholderSourcePath": "./marking_oracle_as_phase/placeholder.qs",
"solutionSourcePath": "./marking_oracle_as_phase/solution.qs",
"solutionDescriptionPath": "./solution.md"
})


### <span style="color:blue">Demo 2.1</span>: Oracle conversion

In this demo we will use your implementation from task 2.1 to convert the marking oracle from task 1.3 to a phase oracle.  Then we will compare this converted oracle to the phase oracle that you implemented in task 1.2.

> *You must have tasks 1.2, 1.3, and 2.1 solved correctly for this demo to work!*

@[example]({"id": "oracle_converter_demo", "codePath": "./oracle_converter_demo.qs"})

> Notice from the above demo that your phase oracle $U_{7,phase}$ behaves the same as the converted version of your marking oracle $U_{7,mark}$, both of which induce a phase flip on the basis state $|111\rangle$!

This way to convert a marking oracle to a phase oracle is useful because many quantum algorithms, such as Grover's search algorithm, rely on a phase oracle, but it is often easier to implement the function as a marking oracle. 
This converter provides a way to implement the function of interest as a marking oracle and then convert it into a phase oracle, which could then be leveraged in a quantum algorithm.

# Part III: Implementing Quantum Oracles

In this section you will implement a few more complicated quantum oracles. 

> Notice that the operation declarations below require adjoint and controlled variants of the oracle to be automatically generated. This is common practice that makes testing and reusing the code easier. Typically Q# compiler will easily generate these variants, as long as you don't use mutable variables or operations that don't support these functors.

### <span style="color:blue">Task 3.1</span>: Implement the OR oracle

**Inputs:**

  1. $N$ qubits in an arbitrary state $|x\rangle$ (input/query register).
  2. A qubit in an arbitrary state $|y\rangle$ (target qubit).

**Goal:**

Flip the state of $|y\rangle$ if the input register is in any basis state
except for $|00...0\rangle$ (the all zero state).

**Examples:**

* If the query register is in the state $|10000001\rangle$, $|11101101\rangle$ or $|0010101\rangle$, flip the state $|y\rangle$.
* If the query register is in the state $|000\rangle$, do nothing.

<br/>
<details>
  <summary><b>Before implementing this oracle, answer the question: are you implementing a marking or a phase oracle?  Click here for the answer!</b></summary>
    This is a marking oracle, because we are flipping the state of the target qubit $|y\rangle$ based on the state of the input $|x\rangle$.
</details>

<br/>
<details>
  <summary><b>Need a hint? Click here</b></summary>
  You need to flip the state of $|y\rangle$ for every input except $|00...0\rangle$, or, alternatively, flip it unconditionally and then flip it for the $|00...0\rangle$ state.   You may find the Q# library function <a href="https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.canon.controlledonint">ControlledOnInt</a> useful in your implementation.
</details>

@[exercise]({
"id": "or_oracle",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./Common.qs"
],
"verificationSourcePath": "./or_oracle/verification.qs",
"placeholderSourcePath": "./or_oracle/placeholder.qs",
"solutionSourcePath": "./or_oracle/solution.qs",
"solutionDescriptionPath": "./solution.md"
})


> Notice that you can modify the state of the input register during your computations (this is what `ControlledOnInt` function does under the hood). 
> However, it is essential to undo those modifications ("uncompute" the changes), except the final one, so that the oracle will preserve the input if it is a basis state.

### <span style="color:blue">Task 3.2</span>: Implement the $k$-th bit oracle

**Inputs:**

  1. $N$ qubits in an arbitrary state $|x\rangle$ (input/query register).
  2. An integer $k$ such that $0 \leq k < N$.

**Goal:**

Flip the sign of the input state $|x\rangle$ if the $k$-th bit of $x$ is $1$.  
**Implement this oracle without using auxiliary qubits.**

**Examples:**

* If the query register is in the state $|010\rangle$ and $k=0$, do nothing.
* If the query register is in the state $|010\rangle$ and $k=1$, flip the sign of the basis state.

<br/>
<details>
  <summary><b>Before implementing this oracle, answer the question: are you implementing a marking or a phase oracle?  Click here for the answer!</b></summary>
    This is a phase oracle, because we are changing the phase of the input state $|x\rangle$ based on the value of the function $f(x)$.
</details>

@[exercise]({
"id": "kth_bit_oracle",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./Common.qs"
],
"verificationSourcePath": "./kth_bit_oracle/verification.qs",
"placeholderSourcePath": "./kth_bit_oracle/placeholder.qs",
"solutionSourcePath": "./kth_bit_oracle/solution.qs",
"solutionDescriptionPath": "./solution.md"
})

> Notice how the oracles - both phase and marking - can take extra "classical" parameters.

Another key tool for implementing quantum oracles is allocating auxiliary qubits to assist in a computation.  Below are some exercises where you will practice that.

### <span style="color:blue">Task 3.3</span>: Implement the OR oracle of all bits except the $k$-th

**Inputs:**

  1. $N$ qubits in an arbitrary state $|x\rangle$ (input/query register).
  2. An integer $k$ such that $0 \leq k < N$.

**Goal:**

Flip the sign of the basis state $|x\rangle$ if any of the bits of $x$ (not considering the $k$-th bit) are $1$ in input register. In other words, the input register with the $k$-th qubit excluded should not be in the all zero state to flip the sign of the input register. The state of the $k$-th qubit does not affect the result.

Feel free to explore implementing this operation with or without auxiliary qubits.

**Examples:**

* If the query register is in the state $|010\rangle$ and $k=0$, flip the sign of the register.
* If the query register is in the state $|010\rangle$ and $k=1$, do nothing.

<br/>
<details>
  <summary><b>Before implementing this oracle, answer the question: are you implementing a marking or a phase oracle?  Click here for the answer!</b></summary>
    This is a phase oracle, because we are changing the phase of the input state $|x\rangle$ based on the value of the function $f(x)$.
</details>

<br/>
<details>
  <summary><b>Need a hint? Click here</b></summary>
  You can reuse the previously implemented oracles and operations, same as how you would use library operations.
  <br/>
  You can use <a href="https://docs.microsoft.com/en-us/azure/quantum/user-guide/language/expressions/itemaccessexpressions">array slicing</a> to get parts of the array before and after the $k$-th element.
</details>

@[exercise]({
"id": "or_but_kth_oracle",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./Common.qs"
],
"verificationSourcePath": "./or_but_kth_oracle/verification.qs",
"placeholderSourcePath": "./or_but_kth_oracle/placeholder.qs",
"solutionSourcePath": "./or_but_kth_oracle/solution.qs",
"solutionDescriptionPath": "./solution.md"
})

# Part IV: More Oracles!  Implementation and Testing:

### <span style="color:blue">Task 4.1</span>: Implement the arbitrary bit pattern oracle

**Inputs:**

  1. $N$ qubits in an arbitrary state $|x\rangle$ (input/query register).
  2. A qubit in an arbitrary state $|y\rangle$ (target qubit).
  3. A boolean array of length $N$ `pattern` representing a basis state; `true` and `false` elements correspond to $|1\rangle$ and $|0\rangle$, respectively.

**Goal:**

Flip the state of $|y\rangle$ if the input register matches the basis state
represented by `pattern`.  

**Examples:**

* If the query register is in the state $|010\rangle$ and `pattern = [false, true, false]`, flip the state $|y\rangle$.
* If the query register is in the state $|1001\rangle$ and `pattern = [false, true, true, false]`, do nothing.
    
<br/>
<details>
  <summary><b>Before implementing this oracle, answer the question: are you implementing a marking or a phase oracle?  Click here for the answer!</b></summary>
    This is a marking oracle, because we are flipping the state of the target qubit $|y\rangle$ based on the state of the input $|x\rangle$.
</details>

<br/>
<details>
  <summary><b>Need a hint? Click here</b></summary>
  You need to flip the state of $|y\rangle$ if $|x\rangle$ matches the given pattern.  You may find the Q# library function <a href="https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.canon.controlledonbitstring">ControlledOnBitString</a> useful in your implementation.
</details>

@[exercise]({
"id": "bit_pattern_oracle",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./Common.qs"
],
"verificationSourcePath": "./bit_pattern_oracle/verification.qs",
"placeholderSourcePath": "./bit_pattern_oracle/placeholder.qs",
"solutionSourcePath": "./bit_pattern_oracle/solution.qs",
"solutionDescriptionPath": "./solution.md"
})

### <span style="color:blue">Task 4.2</span>: Implement the arbitrary bit pattern oracle (challenge version)

**Inputs:**

  1. $N$ qubits in an arbitrary state $|x\rangle$ (input/query register).
  2. A boolean array of length $N$ `pattern` representing a basis state; `true` and `false` elements correspond to $|1\rangle$ and $|0\rangle$, respectively.

**Goal:**
 
Flip the sign of the input state $|x\rangle$ if the input register matches the basis state
represented by `pattern`.  
**Implement this oracle without using auxiliary qubits**

**Examples:**

 * If the query register is in the state $|010\rangle$ and `pattern = [false, true, false]`, flip the sign of the input register.
 * If the query register is in the state $|1001\rangle$ and `pattern = [false, true, true, false]`, do nothing.
  
<br/>
<details>
  <summary><b>Before implementing this oracle, answer the question: are you implementing a marking or a phase oracle?  Click here for the answer!</b></summary>
    This is a phase oracle, because we are changing the phase of the input state $|x\rangle$ based on the value of the function $f(x)$.
</details>

<br/>
<details>
  <summary><b>Need a hint? Click here</b></summary>
  Can you transform the state of the input register based on the <code>pattern</code> value so as to have to flip the phase only for the $|1...1\rangle$ state?
</details>

@[exercise]({
"id": "bit_pattern_challenge",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./Common.qs"
],
"verificationSourcePath": "./bit_pattern_challenge/verification.qs",
"placeholderSourcePath": "./bit_pattern_challenge/placeholder.qs",
"solutionSourcePath": "./bit_pattern_challenge/solution.qs",
"solutionDescriptionPath": "./solution.md"
})

### <span style="color:blue">Task 4.3</span>: Implement the meeting oracle

Suppose that you would like to schedule a meeting with your co-worker Jasmine. 
You both work five day workweeks, and $|x\rangle$ and $|jasmine\rangle$ are 5-bit states represent your and Jasmine's schedules. 
The schedules are indicators of a person being busy on that day: a $1$ bit means that person is busy on that day, and $0$ means they're free for a meeting that day. Implement a function that determines if you and Jasmine can schedule a meeting during the week, i.e., whether there is a day when both schedules have a $0$ simultaneously.

**Inputs:**

  1. 5 qubits in an arbitrary state $|x\rangle$ representing your schedule for the week (input/query register).
  2. 5 qubits in an arbitrary state $|jasmine\rangle$ representing Jasmine's schedule for the week (input/query register).
  3. A qubit in an arbitrary state $|y\rangle$ (target qubit).

**Goal:**

Flip the state of $|y\rangle$ if you and Jasmine are both free on the same day for at least one day during the week.  Recall that a $0$ means that a person is free on that day.

**Examples:**

* If $|x\rangle=|10101\rangle$ and $|jasmine\rangle=|01010\rangle$, do nothing (there is no day on which you both are free).
* If $|x\rangle=|10001\rangle$ and $|jasmine\rangle=|01010\rangle$, flip the state $|y\rangle$ (you are both free on Wednesday).
* If $|x\rangle=|00000\rangle$ and $|jasmine\rangle=|00000\rangle$, flip the state $|z\rangle$ (you are both free all week).
* If $|x\rangle=|11111\rangle$ and $|jasmine\rangle=|11111\rangle$, do nothing (you are both busy all week).
    
<br/>
<details>
  <summary><b>Before implementing this oracle, answer the question: are you implementing a marking or a phase oracle?  Click here for the answer!</b></summary>
    This is a marking oracle, because we are flipping the state of the target qubit $|y\rangle$ based on the state of the inputs $|x\rangle$ and $|jasmine\rangle$. Notice that even though we do not have the typical single-input-register situation that we saw earlier, this is still a marking oracle.
</details>

@[exercise]({
"id": "meeting_oracle",
"codeDependenciesPaths": [
"../KatasLibrary.qs",
"./Common.qs"
],
"verificationSourcePath": "./meeting_oracle/verification.qs",
"placeholderSourcePath": "./meeting_oracle/placeholder.qs",
"solutionSourcePath": "./meeting_oracle/solution.qs",
"solutionDescriptionPath": "./solution.md"
})

### <span style="color:blue">Demo 4.1</span>: Testing an oracle implementation

In this demo we show how you could test an oracle that you've implemented for your own problem. 
For all of the previous oracles that you've implemented, we've been testing your oracle against a reference solution for that task. 
However, if you're designing an oracle for a new problem, you do not have a reference solution for it - if you did, there would be no point for you to program the oracle in the first place!

A good way to test a quantum oracle of interest is to write a classical oracle that performs the same computation classically, and then compare the effect of your quantum oracle on the basis states with the output of the classical oracle for every input (or a lot of the inputs if you are constrained by runtime) to ensure that they match.

Here we will test your implementation from task 4.3 by comparing it to the classical code implementing the same function. 

@[example]({"id": "phase_oracle_alt_bit", "codePath": "./test_meeting_oracle.qs"})

# Part V: What's next?

Thanks for learning with us!  We hope that you enjoyed the tutorial. If you'd like to learn more about implementing quantum oracles, here are some suggestions:

* [Grover's algorithm kata](../../GroversAlgorithm/GroversAlgorithm.ipynb) and [Deutsch-Jozsa algorithm kata](./../DeutschJozsaAlgorithm/DeutschJozsaAlgorithm.ipynb) include simple oracles for you to practice.
* [Marking oracles kata](../../MarkingOracles/MarkingOracles.ipynb) includes practice tasks on more advanced oracles.
* [Solving SAT problems using Grover's algorithm](../../SolveSATWithGrover/SolveSATWithGrover.ipynb) covers implementing oracles for SAT problems.
* [Solving graph coloring problems using Grover's algorithm](../../GraphColoring/GraphColoring.ipynb) covers implementing oracles for graph coloring problem.
* [Solving bounded knapsack problems using Grover's algorithm](../../BoundedKnapsack/BoundedKnapsack.ipynb) covers implementing oracles for bounded knapsack problem.

If you'd like to learn more about quantum algorithms that rely on quantum oracles:
* [Exploring Deutsch-Jozsa algorithm tutorial](https://github.com/microsoft/QuantumKatas/tree/main/tutorials/ExploringDeutschJozsaAlgorithm) introduces the simplest oracle-based algorithm.
* [Exploring Grover’s search algorithm tutorial](https://github.com/microsoft/QuantumKatas/tree/main/tutorials/ExploringGroversAlgorithm) introduces another important quantum algorithm which is used as a build block in many other algorithms.
* [Microsoft Learn module on using Grover's search to solve graph coloring problems](https://docs.microsoft.com/learn/modules/solve-graph-coloring-problems-grovers-search/) provides a detailed example of building a more complicated oracle to solve the graph coloring problem.
