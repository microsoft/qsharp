# Nonlocal Games

@[section]({
    "id": "nonlocal_games__overview",
    "title": "Overview"
})

This kata introduces three quantum nonlocal games that display "quantum pseudo-telepathy" -
the use of quantum entanglement to eliminate the need for classical communication.
In this context, "nonlocal" means that the playing parties are separated by a great distance,
so they cannot communicate with each other during the game.
Another characteristics of these games is that they are "refereed", which means the players try to win against the referee.

**This kata covers the following topics:**

- Clauser, Horne, Shimony, and Hold thought experiment (often abbreviated as CHSH game)
- Greenberger-Horne-Zeilinger game (often abbreviated as GHZ game)
- The Mermin-Peres Magic Square game

**What you should know to start working on this kata:**

- Basic linear algebra
- Basic knowledge of quantum gates and measurements

@[section]({
    "id": "nonlocal_games__chsh_game",
    "title": "CHSH Game"
})

In **CHSH Game**, two players (Alice and Bob) try to win the following game:

Each of them is given a bit (Alice gets X and Bob gets Y), and
they have to return new bits (Alice returns A and Bob returns B)
so that X ∧ Y = A ⊕ B. The trick is, they can not communicate during the game.

> - ∧ is the standard bitwise AND operator.
> - ⊕ is the exclusive or, or XOR operator, so (P ⊕ Q) is true if exactly one of P and Q is true.

To start with, let's take a look at how you would play the classical variant of this game without access to any quantum tools.
Then, let's proceed with quantum strategies for Alice and Bob.

@[exercise]({
    "id": "nonlocal_games__chsh_classical_win_condition",
    "title": "Win Condition",
    "path": "./chsh_classical_win_condition/"
})

@[exercise]({
    "id": "nonlocal_games__chsh_classical_strategy",
    "title": "Alice and Bob's Classical Strategy",
    "path": "./chsh_classical_strategy/"
})

@[exercise]({
    "id": "nonlocal_games__chsh_quantum_alice_strategy",
    "title": "Alice's Quantum Strategy",
    "path": "./chsh_quantum_alice_strategy/"
})

@[exercise]({
    "id": "nonlocal_games__chsh_quantum_bob_strategy",
    "title": "Bob's Quantum Strategy",
    "path": "./chsh_quantum_bob_strategy/"
})

@[section]({
    "id": "nonlocal_games__discussion",
    "title": "Discussion: Probability of Victory for Quantum Strategy"
})

The above quantum strategy adopted by Alice and Bob offers a win rate of $\frac{2 + \sqrt{2}}{4}$, or about 85.36%. Let's see why this is the case.

First, consider the outcome if Alice and Bob simply measure their qubits in the Z basis without manipulating them at all. Because of the entanglement inherent to the Bell state they hold, their measurements will always agree (i.e., both true or both false).
This will suffice for victory in the three scenarios (0,0), (0,1) and (1,0) and fail for (1,1), so their win probability is 75%, the same as that for the straightforward classical strategies of invariably returning both true or both false.

Now let's analyze the optimal quantum strategy.

> As a reminder, probability "wavefunction" for a two-qubit state is given by the following length-4 vector of amplitudes:
> $$\begin{bmatrix}\psi_{00}\\\psi_{01}\\\psi_{10}\\\psi_{11}\end{bmatrix}$$
> $|\psi_{ij}|^2$ gives the probability of observing the corresponding basis state $|ij\rangle$ upon measuring the qubit pair.

The initial state $|00\rangle$ has $\psi_{00} = 1$ and $\psi_{01} = \psi_{10} = \psi_{11} = 0$.
The Bell state we prepare as the first step of the game has an amplitude vector as follows (we'll use decimal approximations for matrix elements):

$$\begin{bmatrix}1/\sqrt{2}\\0\\0\\1/\sqrt{2}\end{bmatrix} = \begin{bmatrix}0.7071\\0\\0\\0.7071\end{bmatrix}$$

Let's analyze the probabilities of outcomes in case of different bits received by players.

## Case 1: Alice holds bit 0

In this case Alice simply measures in the Z basis as above.

- When Bob's bit is 0, he rotates his qubit clockwise by $\pi/8$, which corresponds to the operator

$$\begin{bmatrix}
    0.9239 & 0.3827 & 0 & 0\\
    -0.3827 & 0.9239 & 0 & 0\\
    0 & 0 & 0.9239 & 0.3827\\
    0 & 0 & -0.3827 & 0.9239
\end{bmatrix}$$
This performs the $R_y$ rotation by $\pi/8$ radians clockwise on Bob's qubit while leaving Alice's qubit unchanged.

- If Bob's bit were 1, he would rotate his qubit counterclockwise by $\pi/8$, applying a very similar operator

$$\begin{bmatrix}
    0.9239 & -0.3827 & 0 & 0\\
    0.3827 & 0.9239 & 0 & 0\\
    0 & 0 & 0.9239 & -0.3827\\
    0 & 0 & 0.3827 & 0.9239
\end{bmatrix}$$

Therefore, when Alice has bit 0, the application of the rotation operator to the Bell state gives
$$\begin{bmatrix}
    0.6533 \\
    -0.2706 \\
    0.2706 \\
    0.6533
\end{bmatrix} \text{ or }
\begin{bmatrix}
    0.6533\\
    0.2706\\
    -0.2706\\
    0.6533
\end{bmatrix}$$
depending on whether Bob holds 0 (left-hand case) or 1 (right-hand case).

The result of AND on their input bits will always be 0; thus they win when their outputs agree.  These two cases correspond to the top and bottom elements of the vectors above, with a combined probability of $(0.6533)^2 + (0.6533)^2 = 0.4268 + 0.4268 = 0.8536$, so they have an 85.36% win chance.

## Case 2: Alice holds bit 1

When Alice holds bit 1, she measures in basis X (or, equivalently, Hadamard-transforms her qubit, leaving Bob's be, before making her Z-basis measurement).  This corresponds to applying the operator
$$\begin{bmatrix}
    0.7071 & 0 & 0.7071 & 0\\
    0 & 0.7071 & 0 & 0.7071\\
    0.7071 & 0 & -0.7071 & 0\\
    0 & 0.7071 & 0 & -0.7071
\end{bmatrix}$$
to the Bell state, resulting in a vector of:
$$\begin{bmatrix}
    0.5\\
    0.5\\
    0.5\\
    -0.5
\end{bmatrix}$$

Now, one of the two rotation operators is applied depending on what bit Bob holds, transforming this vector into:
$$\begin{bmatrix}
    0.6533 \\
    0.2706 \\
    0.2706 \\
    -0.6533
\end{bmatrix} \text{ or }
\begin{bmatrix}
    0.2706\\
    0.6533\\
    0.6533\\
    -0.2706
    \end{bmatrix}$$

When Bob holds 0, they still want to return the same parity, which they again do with 85.36% probability (left-hand vector above).
But when Bob holds 1, the AND condition is now true and the players want to answer in opposite parity. This corresponds to the second and third elements of the right-hand vector above.
Thanks to the "magic" of the combination of the counterclockwise rotation and Hadamard transform, they now do this with probability $(0.6533)^2 + (0.6533)^2 = 0.8536$ and thus 85.36% becomes their win odds once more.

## Side notes

- If Bob never rotated his qubit, their entangled state would remain the Bell state if Alice held bit 0 and the state corresponding to $\frac12 \big(|00\rangle + |01\rangle + |10\rangle - |11\rangle\big)$ if Alice held bit 1.

While she and Bob would have a 100% success probability against Alice's 0 bit, they would have only a 50% chance of success if she held bit 1, and thus their win chance would revert to the 75% of the classical strategy again.
- It can be proven that Alice and Bob cannot surpass an overall win probability of 85.36% in the CHSH game. This entails a higher-level discussion of quantum observable theory, for instance see [Tsirelson's bound](https://en.wikipedia.org/wiki/Tsirelson's_bound).

@[section]({
    "id": "nonlocal_games__chsh_e2e",
    "title": "Running CHSH Game End to End"
})

Putting together the building blocks we've implemented into a strategy is very simple:
- Allocate two qubits and prepare a Bell state on them.
- Send one of the qubits to Alice and another to Bob (this step is "virtual", not directly reflected in Q# code, other than making sure that Alice and Bob each act on their qubit only).
- Have Alice and Bob perform their measurements on their respective qubits using `AliceQuantum` and `BobQuantum` operations.
- Release(reset) used qubits
- Return their measurement results.

In the example below you can compare classical and quantum results: first boolean value indicates win for classical Alice and Bob, second boolean is win for quantum Alice and Bob.
You may play with the code and check if there is a difference in results when
1. referee picks non-random bits
2. Bob's qubit is measured first
3. Alice and Bob get not entangled qubit pair

@[example]({"id": "nonlocal_games__chsh_e2edemo", "codePath": "./examples/CHSHGameDemo.qs"})

@[section]({
    "id": "nonlocal_games__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned how to communicate using quantum entanglement in nonlocal quantum games.
