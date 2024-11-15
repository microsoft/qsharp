# Grover's Search Algorithm

@[section]({
    "id": "grovers_search__overview",
    "title": "Overview"
})

This kata introduces you to Grover's search algorithm - one of the fundamental algorithms in quantum computing.

**This kata covers the following topics:**

- The general problem solved by Grover's search algorithm - the search problem
- Implementing Grover's algorithm in Q# for a problem provided as a quantum oracle
- Some of the practical aspects of this algorithm

Note that this tutorial does not cover implementing specific classical functions as quantum oracles in detail. To get familiar with that topic, check out the earlier Oracles and Marking Oracles katas.

**What you should know to start working on this kata:**

- Basic knowledge of quantum gates and measurements
- Basic understanding of quantum oracles

If you need a refresher on these topics, you can check out the previous katas.

@[section]({
    "id": "grovers_search__search_problem",
    "title": "The Search Problem"
})

The problem solved by Grover's search algorithm is called the *search problem* and can be formulated as follows.
You are given a classical function that takes an $n$-bit input and returns a one-bit output $f(x): \{0, 1\}^n \to \{0, 1\}$. 
The task is to find an input $x_0$ for which $f(x_0) = 1$.

Importantly, you don't have any information about the internal structure of the function $f$! It is given to you as a "black box" that allows you to evaluate the value of the function for any input you want, but not to learn anything about how it works.

> This problem is sometimes described as *function inversion*, since it tries to evaluate the inverse of the function $f$.

### Classical solution

If you solve the search problem classically, how many calls to the given function will you need? 

Since you don't know anything about the internal structure of the function, you can't do better than the brute force approach. 
You need to try evaluating the function on different inputs until you either hit the input which produces the desired output or run out of inputs to try and conclude that the desired input doesn't exist. 
This requires $O(2^n)$ function evaluations, since in the worst case scenario you'll need to try all inputs.

### Example problems

Any problem that allows you to check whether a given value of $x$ is a solution to it can be formulated as a search problem,
if you define $f(x)$ as "$1$ if $x$ is a solution to the problem, and $0$ otherwise".

Some of the problems can be formulated as a search problem more naturally than the others.
For example:

- The Boolean satisfiability problem aims to find an assignment of variables in the given Boolean formula 
for which the formula evaluates to true (or to decide that such assignment doesn't exist). 
This is exactly the definition of the search problem, with the input $x$ defined as the set of variables used in the formula and $f(x)$ - as the formula itself.
- Vertex coloring problem aims to find an assignment of colors to the vertices of a given graph that would satisfy the given constraints. In this case, $x$ describes the colors assigned to the vertices, and $f(x)$ is $1$ if the constraints are satisfied or $0$ otherwise.

You will learn more about applying Grover's search to solving specific problems in later katas.
This kata focuses on Grover's search algorithm itself rather than on applying it to a specific problem, so 
it uses a very simple function definition as an example.

Let's define a fixed bit string $p$ of length $P$. The function $f(x)$ is $1$ if the bit string $x$ starts with $p$, and $0$ otherwise.

This function has a very simple implementation and will allow you to experiment with instances of the search problem with different parameters easily.
For example, if the length of the bit string $p$ equals the length of the input, the function $f(x) = 1$ if $x = p$, and $0$ otherwise. In this case, the equation $f(x) = 1$ has exactly one solution, the bit string $p$ itself.

@[exercise]({
    "id": "grovers_search__prefix_oracle",
    "title": "Marking Oracle for Prefix Function",
    "path": "./prefix_oracle/"
})


@[section]({
    "id": "grovers_search__algorithm",
    "title": "Grover's Search Algorithm"
})

### Inputs

You are given the number of bits in the function input $n$ and the phase oracle for the problem we're solving - a "black box" quantum operation $U_f$ that implements a classical function $f(x)$. 

As usual, the phase oracle $U_f$ is defined by its effect on the individual values $x$ (represented as basis states $\ket{x}$). 
If the value of the function on the input $x$ $f(x) = 1$, the corresponding basis state $\ket{x}$ is multiplied by $-1$; otherwise, the basis state is not changed.
Formally, this can be written as follows:

$$U_f \ket{x} = (-1)^{f(x)} \ket{x}$$

> Typically the oracle for Grover's search is implemented as a marking oracle and then converted into a phase oracle using the phase kickback trick.


### Algorithm outline

The high-level outline of the algorithm is very simple:

1. Initialize the quantum system to a well-known starting state.
2. Apply a fixed sequence of "Grover iterations" several times. Each iteration is implemented as pair of operations that includes one call of the oracle "black box".
3. Finally, measuring all qubits will produce the desired output with high probability.

Let's take a closer look at the algorithm.

> We will use a convenient visualization of the algorithm steps rather than mathematical derivation.
> They are equivalent, but the visual representation is much easier to follow.


### Initial state and definitions

Grover's search algorithm begins with a uniform superposition of all the states in the search space.
Typically, the search space is defined as all $n$-bit bit strings, so this superposition is just an even superposition 
of all $N = 2^n$ basis states on $n$ qubits:
$$\ket{\text{all}} = \frac{1}{\sqrt{N}}\sum_{x=0}^{N-1}{\ket{x}} $$

When this superposition is considered in the context of the equation $f(x) = 1$, 
all the basis states can be split in two groups:  "good" (solutions) and "bad" (non-solutions).
If the number of states for which $f(x)=1$ (the number of equation solutions) is $M$, 
two uniform superpositions of "good" and "bad" states can be defined as follows:

$$\ket{\text{good}} = \frac{1}{\sqrt{M}}\sum_{x,f(x)=1}{\ket{x}}$$
$$\ket{\text{bad}} = \frac{1}{\sqrt{N-M}}\sum_{x,f(x)=0}{\ket{x}}$$

Now, the even superposition of all basis states can be rewritten as follows:
$$\ket{\text{all}} = \sqrt{\frac{M}{N}}\ket{\text{good}} + \sqrt{\frac{N-M}{N}}\ket{\text{bad}}$$

The amplutudes $\sqrt{\frac{M}{N}}$ and $\sqrt{\frac{N-M}{N}}$ can then be written in a trigonometric representation,
as a sine and cosine of the angle $\theta$:

$$\sin \theta = \sqrt{\frac{M}{N}}, \cos \theta = \sqrt{\frac{N-M}{N}}$$

With this replacement, the initial state can be written as 

$$\ket{\text{all}} = \sin \theta \ket{\text{good}} + \cos \theta \ket{\text{bad}}$$

The states involved in the algorithm can be represented on a plane on which $\ket{\text{good}}$ and $\ket{\text{bad}}$ vectors correspond to vertical and horizontal axes, respectively.

@[svg]({"path": "./media/1_equal_superposition.svg"})


### Grover's iteration

Each Grover's iteration consists of two operations.

1. The phase oracle $U_f$.
2. An operation called "reflection about the mean".

Applying the phase oracle to the state will flip the sign of all basis states in $\ket{\text{good}}$ 
and leave all basis states in $\ket{\text{bad}}$ unchanged:

$$U_f\ket{\text{good}} = -\ket{\text{good}}$$
$$U_f\ket{\text{bad}} = \ket{\text{bad}}$$

On the circle plot, this transformation leaves the horizontal component of the state vector unchanged and reverses its vertical component. In other words, this operation is a reflection along the horizontal axis.

@[svg]({"path": "./media/2_first_reflection.svg"})

"Reflection about the mean" is an operation for which the visual definition is much more intuitive than the mathematical one.
It is literally a reflection about the state $\ket{\text{all}}$ - the uniform superposition of all basis states in the search space. 

Mathematically, this operation is described as $2\ket{\text{all}}\bra{\text{all}} - I$: it leaves the component of the input state parallel to the state $\ket{\text{all}}$ unchanged and multiplies the component orthogonal to it by $-1$.

@[svg]({"path": "./media/3_second_reflection.svg"})

As we can see, the pair of these reflections combined amount to a counterclockwise rotation by an angle $2\theta$. 
If we repeat the Grover's iteration, reflecting the new state first along the horizontal axis and then along the $\ket{\text{all}}$ vector, it performs a rotation by $2\theta$ again. The angle of this rotation depends only on the angle between the reflection axes and not on the state we reflect!

Each iteration of Grover's search adds $2\theta$ to the current angle in the expression of the system state as a superposition of $\ket{\text{good}}$ and $\ket{\text{bad}}$.
After applying $R$ iterations of Grover's search the state of the system will become

$$\sin{(2R+1)\theta}\ket{\text{good}} + \cos{(2R+1)\theta}\ket{\text{bad}}$$

At first, each iteration brings the state of the system closer to the vertical axis, increasing the probability of measuring one of the basis states that are part of $\ket{\text{good}}$ - the states that are solutions to the problem.

@[exercise]({
    "id": "grovers_search__phase_oracle",
    "title": "Phase Oracle from Marking Oracle",
    "path": "./phase_oracle/"
})

@[exercise]({
    "id": "grovers_search__conditional_phase_flip",
    "title": "Conditional Phase Flip",
    "path": "./conditional_phase_flip/"
})

@[exercise]({
    "id": "grovers_search__reflection_about_state",
    "title": "Reflection about Arbitrary State",
    "path": "./reflection_about_state/"
})


@[section]({
    "id": "grovers_search__iterations",
    "title": "Optimal Number of Iterations"
})

The optimal number of iterations to use in Grover's search algorithm is typically defined as the number of iterations 
after which the success probability of the algorithm - the probability of measuring one of the "good" states - is maximized.

Geometrically, this means that the state vector should be rotated to be as close to the vertical axis as possible.
Mathematically, this means maximizing the amplitude $\sin{(2R+1)\theta}$ of the state $\ket{\text{good}}$ 
in the superposition.
With either definition, the goal is to have the angle $(2R+1)\theta$ that describes the system after $R$ rotations
as close to $\frac{\pi}{2}$ as possible:

$$(2R+1)\theta \approx \frac{\pi}{2}$$

@[svg]({"path": "./media/4_measurement.svg"})

Now, recall that $\theta = \arcsin \sqrt{\frac{M}{N}}$. When $M$ is much smaller than $N$, $\frac{M}{N}$ is close to 0, and $\theta$ is a small angle that can approximated as $\theta \approx \sqrt{\frac{M}{N}}$. This gives the following equation for $R_{opt}$

$$ 2R_{opt}+1 \approx \frac{\pi}{2\theta} = \frac{\pi}{2}\sqrt{\frac{N}{M}}$$
Since $\theta$ is small, $R_{opt}$ is large, and the $+1$ term next to $2R_{opt}$ can be ignored, giving the final formula:
$$ R_{opt} \approx \frac{\pi}{4}\sqrt{\frac{N}{M}}$$

What happens if you use more iterations than is optimal? Each iteration will keep rotating the state $2\theta$ counterclockwise,
getting it further away from the vertical axis and thus reducing the probability of measuring the correct answer.

In this demo you will see how success probability of Grover's algorithm changes when it uses different numbers of iterations
before the final measurements.

@[example]({"id": "grovers_search__e2edemo", "codePath": "./examples/GroversSearchAlgorithmDemo.qs"})


### Verifying that algorithm output is correct

Notice that even when using the optimal number of iterations, you are not guaranteed a $100\%$ success probability.
Grover's search is a probabilistic algorithm, which means that even in the best case it has a non-zero failure probability.
When you use it to solve a problem, you need to check that the output is correct before using it for any purpose.

This can be done classically, if you have access to the classical description of the problem
(in the example used in this kata, you would check that the prefix of the returned state matches the given one.)

In general, the algorithm only gets the marking oracle as an input and doesn't have the information about the classical problem structure. 
However, all information necessary to verify the output is already contained in the oracle itself!  
The effect of the marking oracle on an input, encoded as a basis states of the qubit register, is defined as 
$$U_f \ket{x} \ket{y} = \ket{x} \ket{y \oplus f(x)}$$

This means that if you encode the return of the algorithm $x_0$ as a basis state of the qubit register, 
allocate an extra qubit in the $\ket{0}$ state, and apply the oracle $U_f$ to these qubits, you'll get 
$$U_f \ket{x} \ket{0} = \ket{x} \ket{f(x)}$$

If you measure the last qubit now, you'll get exactly $f(x)$: if it is 1, the algorithm produced a correct answer, otherwise it didn't. If the algorithm failed, you can re-run it from scratch and hopefully get a correct answer on the next attempt!


### Special cases

This calculation for the optimal number of iterations is done under the assumption that $M$ is much smaller than $N$ but greater than $0$. In other words, the algorithm works if solutions to the search problem exist, but there are very few of them.

What happens if these assumptions are not valid?

#### No solutions ($M = 0$)

In this case the starting system state  $\ket{\psi} = \ket{\text{bad}}$, and $\theta = \arcsin \sqrt{\frac{M}{N}} = 0$.
No matter how many iterations we do, the probability of our measurement yielding a marked state is $0$.

In practice this means that Grover's search will yield a random non-solution every time. 
To detect that this is the case, we need to run the algorithm multiple times and note the results. If none of them are problem solutions, we can conclude that the problem doesn't have a solution.

#### Solutions make up half of the search space

If $M = \frac{N}{2}$, then $\theta = \arcsin \sqrt\frac{N/2}{N}  = \arcsin \sqrt\frac{1}{2} = \frac{\pi}{4}$.   
This means that after an arbitrary number of iterations $R$ the amplitude of the basis state $\ket{\text{good}}$ in the  system will be:

$$\sin{(2R+1)\theta} = \sin\frac{(2R+1)\pi}{4} = \pm \frac{1}{\sqrt{2}}$$

The probability of the measurement yielding a solution is then $P(\ket{\text{good}}) = \sin^2\frac{(2R+1)\theta}{2} = (\pm \frac{1}{\sqrt{2}})^2 = \frac{1}{2}$

You can see that the probability of measuring a state that is a solution remains constant regardless of the number of iterations.

#### Solutions make up more than half of the search space

If $\frac{N}{2} < M \leq N$, then $\frac{\pi}{4} < \theta \leq \frac{\pi}{4}$. 
Now using even one iteration doesn't always increase $P(\ket{\text{good}}) = \sin^2{(2R+1)\theta}$.
In fact, the first iteration is likely to decrease the probability of success!

> Have you ever wondered why all tutorials on Grover's search start with two-bit functions? 
> That's the reason why: if you have only one bit, you only have functions for which $M=0$, $M=\frac{N}{2}$, or $M=N$, and none of these make for a good illustration of the algorithm!

The last two scenarios are a lot easier to handle classically. 
Indeed, a randomly selected classical value has a probability of being a problem solution $p = \frac{M}{N} > \frac{1}{2}$! 
If we repeat the random selection of the variable $k$ times, the probability of success grows to $1-(1-p)^k$, thus by increasing $k$ we can get this probability as close to $1$ as we need.
For example, For $p=0.5$ and $k=10$ the probability of success is about $99.9\%$.

#### Unknown number of solutions

In practical applications you don't usually know how many solutions your problem has before you start solving it. 
In this case, you can pick the number of iterations as a random number between 1 and $\frac{\pi}{4} \sqrt{N}$, 
and if the search did not yield the result on the first run, re-run it with a different number of iterations. 
Since Grover's algorithm is probabilistic, you don't nee to get the exact number of iterations to get a correct answer!


@[section]({
    "id": "grovers_search__practicality",
    "title": "Practicality of Grover's Search Algorithm"
})

You saw that Grover's search algorithm offers a theoretical advantage over the classical brute force approach.
Does this mean that it is an algorithm that will offer us a practical advantage for search problems?

Unfortunately, there are several reasons why the answer is "no".

First, Grover's algorithm uses no information about problem structure, and demonstrates the advantage over the classical algorithm under the assumption that the classical algorithm doesn't use this information either. 
However, the best classical algorithms exploit problem structure, allowing solutions that are much better than brute force search. 
Compared to these algorithms, Grover's algorithm often doesn't offer even a theoretical advantage.
Additionally, classical algorithms can use parallel processing and benefit from getting multiple computation results at once.

Implementing the quantum oracle which encodes a problem instance on a quantum computer can be hard - both in terms of coming up with the code that does that and in terms of the execution time of a single oracle call. The advantage offered by Grover's algorithm is described in terms of the number of queries to the oracle and classical function evaluations. But, if a single oracle call takes many orders of magnitude longer to run than a single classical function evaluation, this overhead can easily negate the advantage in the number of calls. This limitation comes into play even for problems which don't have efficient classical algorithms, such as hash inversion.

This means that Grover's search is unlikely to offer us a practical advantage over the best classical algorithms for any problems. 
However, it remains one of the several fundamental quantum computing algorithms, and a great educational algorithm.
Besides, a technique that generalizes the idea behind Grover's algorithm called *amplitude amplification* can be used 
as a building block for other quantum algorithms.


@[section]({
    "id": "grovers_search__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned Grover's search algorithm - one of the fundamental algorithms in quantum computing.
Here are a few key concepts to keep in mind:

- Grover's search algorithm is an algorithm for unstructured search, also known as function inversion.
- This algorithm offers a theoretical advantage over classical brute force search algorithm.
- Grover's algorithm is unlikely to offer practical advantage for any problems, since classical algorithms typically rely on the knowledge of problem structure and thus do much better than brute force search.

Next, you will practice applying Grover's algorithm to several more interesting problems.
