# Deutsch-Jozsa and Bernstein-Vazirani Algorithms

@[section]({
    "id": "deutsch_jozsa__overview",
    "title": "Overview"
})

This kata introduces you to Deutsch-Jozsa algorithm - one of the most famous algorithms in quantum computing. The problem it solves has little practical value, but the algorithm itself is one of the earliest examples of a quantum algorithm that is exponentially faster than any possible deterministic algorithm for the same problem. It is also relatively simple to explain and illustrates several very important concepts (such as quantum oracles). As such, Deutsch–Jozsa algorithm is part of almost every introductory course on quantum computing.

**This kata covers the following topics:**

- The problem solved by Deutsch-Jozsa algorithm and the classical solution to it
- Multi-qubit phase oracles (for a more detailed introduction to phase oracles, see Oracles kata)
- Deutsch-Jozsa algorithm
- Implementing oracles and end-to-end Deutsch-Jozsa algorithm in Q#
- Bernstein-Vazirani algorithm and the problem solved by it

**What you should know to start working on this kata:**

- Basic single-qubit gates
- Quantum measurements
- Deutsch algorithm - the single-qubit variant of Deutsch-Jozsa algorithm. 

If you need a refresher on these topics, you can check out the previous katas.

@[section]({
    "id": "deutsch_jozsa__problem",
    "title": "The Problem"
})

You are given a classical function that takes an $N$-bit string as an input and returns one bit: $f(x): \{0, 1\}^N \to \{0, 1\}$. You are guaranteed that the function $f$ is

- either *constant* (returns the same value for all inputs) 
- or *balanced* (returns value $0$ for half of the inputs and $1$ for the other half of the inputs). 

The task is to figure out whether the function is constant or balanced.

**Examples**

- $f(x) \equiv 0$ or $f(x) \equiv 1$ are constant functions (and they're actually the only constant functions in existence).
- $f(x) = x \bmod 2$ (the least significant bit of $x$) or $f(x) = 1 \text{ if the binary notation of }x \text{ has odd number of 1s and 0 otherwise}$ are examples of multi-bit balanced functions. Indeed, for both these functions you can check that for every possible input $x$ for which $f(x) = 0$ there exists an input $x^\prime$ (equal to $x$ with the least significant bit flipped) such that $f(x^\prime) = 1$, and vice versa, which means that the function is balanced.

If you solve this problem classically, how many calls to the given function will you need? 

The first call will give you no information - regardless of whether it returns $0$ or $1$, the function could still be constant or balanced.
In the best case scenario, the second call will return a different value. You'll be able to conclude that the function is balanced in just $2$ calls. 
However, if you get the same value for the first two calls, you'll have to keep querying the function until either the function returns a different value, or until you perform $2^{N-1}+1$ queries that return the same value - only in this case will you know with certainty that the function is constant.

What about the quantum scenario?


@[section]({
    "id": "deutsch_jozsa__oracles",
    "title": "Multi-Qubit Oracles"
})

In the quantum scenario, the classical function you're working with is implemented as a quantum oracle - a "black box" operation used as input to another algorithm. This operation is implemented in a way which allows you to perform calculations not only on individual inputs, but also on superpositions of inputs. 

To enable the oracle to act on quantum states instead of classical values, the integer input $x$ is represented in binary $x = (x_{0}, x_{1}, \dots, x_{N-1})$, 
and encoded into an $N$-qubit register: $\ket{\vec{x} } = \ket{x_{0} } \otimes \ket{x_{1} } \otimes \cdots \otimes \ket{x_{N-1} }$.
The phase oracle $U_f$ for this function is defined as follows:

$$U_f \ket{\vec{x} } = (-1)^{f(x)} \ket{\vec{x} }$$

The function $f$ can return only two values, 0 or 1, which result in no phase change or multiplication by a relative phase $-1$, respectively.

The effect of such an oracle on any single basis state isn't particularly interesting: it just adds a global phase which isn't something you can observe. However, if you apply this oracle to a *superposition* of basis states, its effect becomes noticeable. 
Remember that quantum operations are linear: if you define the effect of an operation on the basis states, you'll be able to deduce its effect on superposition states (which are just linear combinations of the basis states) using its linearity.

Let's see how to implement several examples of multi-bit constant and balanced functions as phase oracles in Q#.

1. $f(x) \equiv 0$

This is the easiest function to implement: if $f(x) \equiv 0$, 

$$U_f \ket{x} \equiv (-1)^0 \ket{x} = \ket{x}$$

This means that $U_f$ is an identity - a transformation which does absolutely nothing! 

2. $f(x) \equiv 1$

The second constant function is slightly trickier: if $f(x) \equiv 1$

$$U_f \ket{x} \equiv (-1)^1 \ket{x} = - \ket{x}$$

Now $U_f$ is a negative identity, that is, a transformation which applies a global phase of $-1$ to the state. 
A lot of algorithms just ignore the global phase accumulated in them, since it isn't observable. 
However, if you want to be meticulous, you can use the $R$ gate which performs a given rotation around the given axis. 
When called with `PauliI` axis, this operation applies a global phase to the given qubit. 
You can use any qubit to apply this gate, for example, `qs[0]`.

3. $f(x) = x \bmod 2$

The binary representation of $x$ is $x = (x_{0}, x_{1}, \dots, x_{N-1})$, with the least significant bit encoded in the last bit (stored in the last qubit of the input array). Then you can rewrite the function as

$$f(x) = x_{N-1}$$

Let's use this in the oracle effect expression:

$$U_f \ket{x} = (-1)^{f(x)} \ket{x} = (-1)^{x_{N-1}} \ket{x} = \ket{x_{0} } \otimes \cdots \otimes \ket{x_{N-2} } \otimes (-1)^{x_{N-1}} \ket{x_{N-1}}$$

This means that you only need to use the last qubit in the implementation: do nothing if it's $\ket{0}$ and apply a phase of $-1$ if it's $\ket{1}$. This is exactly the effect of the $Z$ gate!

You can write out the oracle unitary as follows:

$$U_f = \mathbb{1} \otimes \cdots \otimes \mathbb{1} \otimes Z$$

In the following demo you'll see how to implement three multi-bit functions as quantum oracles, and their effect on a quantum state.
After that, you'll try to implement the oracles for two more functions on your own!

@[example]({"id": "deutsch_jozsa__oracle_implementations", "codePath": "./examples/OracleImplementationDemo.qs"})

@[exercise]({
    "id": "deutsch_jozsa__msb_oracle",
    "title": "Oracle for f(x) = most significant bit of x",
    "path": "./msb_oracle/"
})

@[exercise]({
    "id": "deutsch_jozsa__parity_oracle",
    "title": "Oracle for f(x) = parity of the number of 1 bits in x",
    "path": "./parity_oracle/"
})


@[section]({
    "id": "deutsch_jozsa__algorithm",
    "title": "Solving the Problem: Deutsch-Jozsa Algorithm"
})

Now let's return to the problem of figuring out whether the given function is constant or balanced.
The following sections present the algorithm in detail step-by-step.

### Inputs

You are given the number of bits in the oracle input $N$ and the oracle itself - a "black box" operation $U_f$ that implements a classical function $f(x)$. You are guaranteed that the function implemented by the oracle is either constant or balanced.

### The starting state

The algorithm starts with $N$ qubits in the $\ket{0...0} = \ket{0}^{\otimes N}$ state.

### Step 1. Apply Hadamard transform to each qubit

Applying the $H$ gate to one qubit in the $\ket{0}$ state converts it to the $\frac{1}{\sqrt2} \big(\ket{0} + \ket{1} \big)$ state, which is an equal superposition of both basis states on one qubit. 

If you apply the $H$ gate to each of the two qubits in the $\ket{00}$ state, you get 

$$(H \otimes H) \ket{00} = \big(H \ket{0} \big) \otimes \big(H \ket{0}\big) = \left(\frac{1}{\sqrt2} \big(\ket{0} + \ket{1} \big)\right) \otimes \left(\frac{1}{\sqrt2} \big(\ket{0} + \ket{1} \big)\right) = \frac{1}{2} \big(\ket{00} + \ket{01} + \ket{10} + \ket{11} \big)$$

This is just an equal superposition of all basis states on two qubits! 
You can extend the same thinking to applying the $H$ gate to each of the $N$ qubits in the $\ket{0...0}$ state to conclude that this transforms them into a state that is an equal superposition of all basis states on $N$ qubits.

Mathematically, the transformation "apply $H$ gate to each of the $N$ qubits" can be denoted as $H^{\otimes N}$. After applying this transformation, you get the following state:

$$H^{\otimes N} \ket{0}^{\otimes N} = \big( H\ket{0} \big)^{\otimes N} = \left( \frac{1}{\sqrt2} \big(\ket{0} + \ket{1} \big) \right)^{\otimes N} = \frac{1}{\sqrt{2^N}} \sum_{x=0}^{2^N-1} \ket{x}$$


### Step 2. Apply the oracle

This step is the only step in which you use the knowledge of the classical function, given as the quantum oracle. 
This step keep the amplitudes of the basis states for which $f(x) = 0$ unchanged, and multiply the amplitudes of the basis states for which $f(x) = 1$ by $-1$.

Mathematically, the results of oracle application can be written as follows:

$$U_f \left(\frac{1}{\sqrt{2^N}} \sum_{x=0}^{2^N-1} \ket{x} \right) = \frac{1}{\sqrt{2^N}} \sum_{x=0}^{2^N-1} U_f\ket{x} = \frac{1}{\sqrt{2^N}} \sum_{x=0}^{2^N-1} (-1)^{f(x)} \ket{x}$$

### Step 3. Apply Hadamard transform to each qubit again

In this step, you don't need to worry about the whole expression for the state of the qubits after applying the $H$ gates to them; it's enough to calculate only the resulting amplitude of the basis state $\ket{0}^{\otimes N}$.

Consider one of the basis states $\ket{x}$ in the expression $\sum_{x=0}^{2^N-1} (-1)^{f(x)} \ket{x}$.  
It can be written as $\ket{x} = \ket{x_{0} } \otimes \cdots \otimes \ket{x_{N-1}}$, where each $\ket{x_k}$ is either $\ket{0}$ or $\ket{1}$.  
When you apply the $H$ gates to $\ket{x}$, we'll get $H^{\otimes N} \ket{x} = H\ket{x_{0} } \otimes \cdots \otimes H\ket{x_{N-1}}$, where each term of the tensor product is either $H\ket{0} = \frac{1}{\sqrt2}\big(\ket{0} + \ket{1} \big) = \ket{+}$ or $H\ket{1} = \frac{1}{\sqrt2}\big(\ket{0} - \ket{1} \big) = \ket{-}$. 
If you open the brackets in this tensor product, you get a superposition of all $N$-qubit basis states, each of them with amplitude $\frac{1}{\sqrt{2^N}}$ or $-\frac{1}{\sqrt{2^N}}$ — and, since the amplitude of the $\ket{0}$ state in both $\ket{+}$ and $\ket{-}$ is positive, you know that the amplitude of the basis state $\ket{0}^{\otimes N}$ ends up positive, that is, $\frac{1}{\sqrt{2^N}}$.

Now you can calculate the amplitude of the $\ket{0}^{\otimes N}$ state in the expression $H^{\otimes N} \left( \frac{1}{\sqrt{2^N}} \sum_{x=0}^{2^N-1} (-1)^{f(x)} \ket{x} \right)$: in each of the $2^N$ terms of the sum its amplitude is $\frac{1}{\sqrt{2^N}}$. Therefore, you get the total amplitude

$$\frac{1}{\sqrt{2^N}} \sum_{x=0}^{2^N-1} (-1)^{f(x)} \frac{1}{\sqrt{2^N}} = \frac{1}{2^N} \sum_{x=0}^{2^N-1} (-1)^{f(x)}$$

### Step 4. Perform measurements and interpret the result

So far, you didn't use the fact that the function you are given is constant or balanced. Let's see how this affects the amplitude of the $\ket{0}^{\otimes N}$ state.

* If the function is constant, $f(x) = C$ (either always $0$ or always $1$), you get  
  $$\frac{1}{2^N} \sum_{x=0}^{2^N-1} (-1)^{f(x)} = \frac{1}{2^N} \sum_{x=0}^{2^N-1} (-1)^{C} = \frac{1}{2^N} \cdot 2^N (-1)^C = (-1)^C$$
  Since the sum of squares of amplitudes of all basis states always equals $1$, the amplitudes of the rest of the basis states have to be 0 - this means that the state of the qubits after step 3 *is* $\ket{0}^{\otimes N}$.

* If the function is balanced, that is, returns $0$ for exactly half of the inputs and $1$ for the other half of the inputs, exactly half of the terms in the sum $\frac{1}{2^N} \sum_{x=0}^{2^N-1} (-1)^{f(x)}$ will be $1$ and the other half of the terms will be $-1$, and they will all cancel out, leaving the amplitude of $\ket{0}^{\otimes N}$ equal to $0$.

Now, what happens when you measure all qubits? (Remember that the probability of getting a certain state as a result of measurement equals to the square of the amplitude of this state.)

If the function is constant, the only measurement result you can get is all zeros - the probability of getting any other result is $0$. If the function is balanced, the probability of getting all zeros is $0$, so you'll get any measurement result except this.

This is exactly the last step of the algorithm: **measure all qubits, if all measurement results are 0, the function is constant, otherwise it's balanced**.

### Summary

In the end, the algorithm is very straightforward:

1. Apply the $H$ gate to each qubit.
2. Apply the oracle.
3. Apply the $H$ gate to each qubit again.
4. Measure all qubits.
5. If all qubits are measured in $\ket{0}$ state, the function is constant, otherwise it's balanced.

Note that this algorithm requires only $1$ oracle call, and always produces the correct result!

@[exercise]({
    "id": "deutsch_jozsa__implement_dj",
    "title": "Implement Deutsch-Jozsa Algorithm",
    "path": "./implement_dj/"
})


@[section]({
    "id": "deutsch_jozsa__e2e",
    "title": "Running Deutsch-Jozsa Algorithm End to End"
})

The last demo in this kata shows you how to combine the oracles you've seen so far and the Deutsch-Jozsa algorithm you've implemented into an end-to-end application that will check whether each oracle implements a constant or a balanced function.

@[example]({"id": "deutsch_jozsa__e2edemo", "codePath": "./examples/DeutschJozsaAlgorithmDemo.qs"})


@[section]({
    "id": "deutsch_jozsa__bernstein-vazirani",
    "title": "Bernstein-Vazirani Algorithm"
})

To wrap up the discussion in this kata, let's take a look at a problem solved using a similar approach - the Bernstein-Vazirani algorithm.
In this problem, you are also given an oracle implementing an $N$-bit function $f(x): \{0, 1\}^N \to \{0, 1\}$.
However, this time the function is guaranteed to be a *scalar product function*, that is, there exists an $N$-bit string $s$
that allows the following representation ($\cdot$ is bitwise inner product of integers modulo $2$):

$$f(x) = x \cdot s = \sum_{k=0}^{N-1} x_k s_k \bmod 2$$

The task is to recover the hidden bit string $s$.

**Examples**

- $f(x) \equiv 0$ is an example of such a function with $s = 0, \dots, 0$.
- $f(x) = 1 \text{ if x has odd number of 1s, and } 0 \text{ otherwise }$ is another example of such a function, with $s = 1, \dots, 1$.

If you solve this problem classically, how many calls to the given function will you need? 
You'd need to use one query to recover each bit of $s$ (the query for $k$-th bit can be a bit string with $1$ in the $k$-th bit and zeros in all other positions), for a total of $N$ queries.

What about the quantum scenario?
It turns out that the algorithm that allows you to solve this problem looks just like Deutsch-Jozsa algorithm, 
except for the way you interpret the measurement results on the last step. To see this, you'll need to take another look 
at the math involved in applying Hadamard gates to multiple qubits.

### Apply Hadamard transform to each qubit: a different view

When you apply an $H$ gate to a single qubit in the basis state $\ket{x}$, you can write the result as the following sum:

$$H\ket{x} = \frac1{\sqrt2} (\ket{0} + (-1)^{x} \ket{1}) = \frac1{\sqrt2} \sum_{z \in {0, 1}} (-1)^{x \cdot z} \ket{z}$$

If you use this representation to spell out the result of applying an $H$ gate to each qubit of an $N$-qubit basis state 
$\ket{x} = \ket{x_0}\ket{x_1} \dots \ket{x_{N-1}}$, you get:

$$H\ket{x} = \frac1{\sqrt{2^N}} \sum_{z_k \in {0, 1}} (-1)^{x_0z_0 + \dots + x_{N-1}z_{N-1}} \ket{z_0}\ket{z_1} \dots \ket{z_{N-1}} =$$

$$= \frac1{\sqrt{2^N}} \sum_{z = 0}^{2^N-1} (-1)^{x \cdot z} \ket{z}$$

With this in mind, let's revisit the algorithm and see how you can write the exact quantum state after it.

### Bernstein-Vazirani algorithm

Bernstein-Vazirani algorithm follows the same outline as Deutsch-Jozsa algorithm:

1. Apply the $H$ gate to each qubit.
2. Apply the oracle.
3. Apply the $H$ gate to each qubit again.
4. Measure all qubits.

You know that after the second step the qubits end up in the following state:

$$\frac{1}{\sqrt{2^N}} \sum_{x=0}^{2^N-1} (-1)^{f(x)} \ket{x}$$

Now, once you apply the Hadamard gates to each qubit, the system state becomes:

$$\frac{1}{\sqrt{2^N}} \sum_{x=0}^{2^N-1} \sum_{z=0}^{2^N-1} (-1)^{f(x) + x \cdot z} \ket{z}$$

> In Deutsch-Jozsa algorithm, you looked at the amplitude of the $\ket{0}$ state in this expression, which was 
> $\frac{1}{\sqrt{2^N}} \sum_{x=0}^{2^N-1} (-1)^{f(x)}$.

Now, let's take a look at the amplitude of the $\ket{s}$ state - the state that encodes the hidden bit string you're looking for.
This amplitude is 

$$\frac{1}{\sqrt{2^N}} \sum_{x=0}^{2^N-1} (-1)^{f(x) + x \cdot s}$$

Since $f(x) = x \cdot s$, for all values of $x$ $f(x) + x \cdot s = 2 x \cdot s$, and $(-1)^{f(x) + x \cdot s} = 1$.
Overall the amplitude of $\ket{s}$ is 

$$\frac{1}{\sqrt{2^N}} \sum_{x=0}^{2^N-1} 1 = \frac{1}{\sqrt{2^N}} 2^N = 1$$

This means that the state after applying the Hadamard gates is just $\ket{s}$, and measuring it gives you the bit string $s$!
And, same as Deutsch-Jozsa algorithm, Bernstein-Vazirani algorithm takes only one oracle call.

@[exercise]({
    "id": "deutsch_jozsa__implement_bv",
    "title": "Implement Bernstein-Vazirani Algorithm",
    "path": "./implement_bv/"
})


@[section]({
    "id": "deutsch_jozsa__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned Deutsch-Jozsa and Bernstein-Vazirani algorithms.

- Deutsch-Jozsa algorithm is the simplest example of a quantum algorithm that is exponentially faster than any possible deterministic algorithm for the same problem.
- Bernstein-Vazirani algorithm is a similar algorithm that extracts information about the hidden bit string of the given function that is known to be a scalar product function. It offers a linear speedup compared to a classical algorithm for the same problem.
- Quantum oracles don't allow you to evaluate the function on all inputs at once! Instead, Deutsch-Jozsa algorithm finds a clever way to aggregate information about all function values into a few bits that indicate whether they are all the same or not. Bernstein-Vazirani algorithm uses a similar approach to encode the information about the hidden bit string into the state of the qubits at the end of the algorithm.
