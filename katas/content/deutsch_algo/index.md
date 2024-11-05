# Deutsch Algorithm

@[section]({
    "id": "deutsch_algo__overview",
    "title": "Overview"
})

This kata introduces you to Deutsch algorithm - the single-qubit variant of Deutsch–Jozsa algorithm, one of the most famous educational algorithms in quantum computing.

**This kata covers the following topics:**

- The problem solved by Deutsch algorithm and the classical solution to it
- Single-qubit phase oracles (for a more detailed introduction to phase oracles, see Oracles kata)
- Deutsch algorithm
- Implementing oracles and end-to-end Deutsch algorithm in Q#

**What you should know to start working on this kata:**

- Basic single-qubit gates
- Quantum measurements

If you need a refresher on these topics, you can check out the previous katas.

@[section]({
    "id": "deutsch_algo__problem",
    "title": "The Problem"
})

You are given a classical function that takes one bit as an input and returns one bit: $f(x): \{0, 1\} \to \{0, 1\}$. You are guaranteed that the function $f$ is

- either *constant* (returns the same value for all inputs) 
- or *variable* (returns different values for different inputs). 

The task is to figure out whether the function is constant or variable. In other words, you need to decide whether $f(0) = f(1)$ (which is the same as the function being constant for single-bit functions).

**Examples**

- $f(x) \equiv 0$ or $f(x) \equiv 1$ are constant functions (and they are actually the only constant functions in existence).
- $f(x) = x$ and $f(x) = 1 - x$ are the only variable functions (and they are the only variable functions for single-bit functions).

If you solve this problem classically, how many calls to the given function will you need? 

The first function call will give you no information - regardless of whether it returns $0$ or $1$, the function could still be constant or variable.
You'll need to call the function a second time to evaluate its return values for both possible inputs to be able to check whether these values are equal.
This means that the classical solution requires **2** function calls.

What about the quantum scenario?


@[section]({
    "id": "deutsch_algo__oracles",
    "title": "Single-Qubit Oracles"
})

In the quantum scenario, the classical function you're working with is implemented as a quantum oracle - a "black box" operation used as input to another algorithm. This operation is implemented in a way which allows to perform calculations not only on individual inputs, but also on superpositions of inputs. 

The oracle has to act on quantum states instead of classical values. 
To enable this, integer input $x$ is represented as a qubit state $\ket{x}$.

The type of oracles used in this tutorial are called *phase oracles*. A phase oracle $U_f$ encodes the value of the classical function $f$ it implements in the phase of the qubit state as follows:

$$U_f \ket{x} = (-1)^{f(x)} \ket{x}$$

In this case $f$ can return only two values, 0 or 1, which result in no phase change or multiplication by a relative phase $-1$, respectively.

The effect of such an oracle on any single basis state isn't particularly interesting: it just adds a global phase which isn't something you can observe. However, if you apply this oracle to a *superposition* of basis states, its effect becomes noticeable. 
Remember that quantum operations are linear: if you define the effect of an operation on the basis states, you'll be able to deduce its effect on superposition states (which are just linear combinations of the basis states) using its linearity.

There are only four single-bit functions, so you can see how to implement them all as phase oracles in Q#.

1. $f(x) \equiv 0$

This is the easiest function to implement: if $f(x) \equiv 0$, 

$$U_f \ket{x} \equiv (-1)^0 \ket{x} = \ket{x}$$

This means that $U_f$ is an identity - a transformation which does absolutely nothing! 

2. $f(x) \equiv 1$

The second constant function is slightly trickier: if $f(x) \equiv 1$

$$U_f \ket{x} \equiv (-1)^1 \ket{x} = - \ket{x}$$

Now $U_f$ is a negative identity, that is, a transformation which applies a global phase of $-1$ to the state. 
A lot of algorithms just ignore the global phase accumulated in them, since it isn't observable. 
However, if you want to be really meticulous, you can use the $R$ gate which performs a given rotation around the given axis. 
When called with `PauliI` axis, this operation applies a global phase to the given qubit.

3. $f(x) = x$

$$U_f \ket{x} = (-1)^{f(x)} \ket{x} = (-1)^{x} \ket{x}$$

This means that you don't need to do anything if the qubit is in the $\ket{0}$ state, and apply a phase of $-1$ if it is in the $\ket{1}$ state. This is exactly the effect of the $Z$ gate!

In the following demo, you'll see how to implement the first three one-bit functions as quantum oracles, and their effect on a qubit state.
After that, you'll try to implement the oracle for the fourth function on your own!

@[example]({"id": "deutsch_algo__oracle_implementations", "codePath": "./examples/OracleImplementationDemo.qs"})

@[exercise]({
    "id": "deutsch_algo__one_minus_x_oracle",
    "title": "Oracle for f(x) = 1 - x",
    "path": "./one_minus_x_oracle/"
})


@[section]({
    "id": "deutsch_algo__algorithm",
    "title": "Solving the Problem: Deutsch Algorithm"
})

Now let's return to the problem of figuring out whether the given function is constant or variable for single-bit functions.
What can we do if we are given a quantum oracle $U_f$ implementing the function $f(x)$?

There are two possible inputs to the function, $\ket{0}$ and $\ket{1}$. Let's see what happens if you apply the oracle to their superposition:

$$U_f \left( \frac{1}{\sqrt2} \big( \ket{0} + \ket{1} \big) \right) 
= \frac{1}{\sqrt2} \big( U_f \ket{0} + U_f \ket{1} \big) 
= \frac{1}{\sqrt2} \big( (-1)^{f(0)} \ket{0} + (-1)^{f(1)} \ket{1} \big)$$.

- If $f(0) = f(1)$, the relative phases of the two basis states are the same, and the resulting state is $\ket{+} = \frac{1}{\sqrt2} \big( \ket{0} + \ket{1} \big)$ (up to a global phase). 
- If $f(0) \neq f(1)$, the relative phases of the two basis states differ by a factor of $-1$, and the resulting state is $\ket{-} = \frac{1}{\sqrt2} \big( \ket{0} - \ket{1} \big)$ (up to a global phase). 

Now, the states $\ket{+}$ and $\ket{-}$ can be distinguished using measurement: if you apply the H gate to each of them, you'll get $H\ket{+} = \ket{0}$ if $f(0) = f(1)$, or $H\ket{-} = \ket{1}$ if $f(0) \neq f(1)$. This means that one oracle call doesn't let you calculate both $f(0)$ and $f(1)$, but it allows you to figure out whether $f(0) = f(1)$!

Overall, the algorithm is very straightforward:

1. Start with a qubit in the $\ket{0}$ state.
2. Apply the $H$ gate to the qubit.
3. Apply the oracle.
4. Apply the $H$ gate to the qubit again.
5. Measure the qubit: if it's in the $\ket{0}$ state, the function is constant, otherwise it's variable.

Note that this algorithm requires only **1** oracle call, and always produces the correct result (the algorithm is deterministic).

@[section]({
    "id": "deutsch_algo__visualization",
    "title": "Visualizing Deutsch Algorithm"
})

You can follow the steps of the algorithm for the constant and the balanced scenarios using a neat visualization. Since Deutsch algorithm deals only with states with real amplitudes, you can map all states on the unit circle, and follow the state evolution through the steps.

1. Start with a qubit in the $\ket{0}$ state and apply the $H$ gate to the qubit.
   <br/>
   @[svg]({"path": "./media/Plus_state.svg"})

2. Apply the oracle.  
   Here, the difference between the two scenarios becomes noticeable. In the constant scenario, $\ket{0}$ and $\ket{1}$ states get the same phase (either $1$ or $-1$), so the state remains the same or acquires a global phase of $-1$, which is physically the same state. In the variable scenario, zero and one states get different phases, so the state changes!
   <br/>
   @[svg]({"path": "./media/Apply_oracle.svg"})

3. Apply the $H$ gate to the qubit again.
   Now, you get the $\ket{0}$ state for both constant scenarios and the $\ket{1}$ state for both variable scenarios!
   <br/>
   @[svg]({"path": "./media/Apply_hadamard.svg"})


@[exercise]({
    "id": "deutsch_algo__implement_algo",
    "title": "Implement Deutsch Algorithm",
    "path": "./implement_algo/"
})


@[section]({
    "id": "deutsch_algo__e2e",
    "title": "Running Deutsch Algorithm End to End"
})

The last demo in this kata shows you how to combine the oracles you've seen so far and the Deutsch algorithm you've implemented into an end-to-end application that will check whether each oracle implements a constant or a variable function.

@[example]({"id": "deutsch_algo__e2edemo", "codePath": "./examples/DeutschAlgorithmDemo.qs"})


@[section]({
    "id": "deutsch_algo__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned Deutsch algorithm.

- Deutsch algorithm is the smallest example of a quantum algorithm that allows to answer a question about a function in fewer queries than its classical counterpart: one query to a quantum oracle versus two queries to a classical function.
- Quantum oracles don't allow you to evaluate the function on all inputs at once! Instead, Deutsch algorithm finds a clever way to aggregate information about both function values into a single bit that indicates whether they are equal or not.

Next, you will learn about the more general case of this problem and the algorithm to solve it in the Deutsch-Jozsa Algorithm kata.
