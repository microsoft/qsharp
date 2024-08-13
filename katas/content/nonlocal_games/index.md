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
so that X $\land$ Y = A $\oplus$ B. The trick is, they can not communicate during the game.

> - $\land$ is the standard bitwise AND operator.
> - $\oplus$ is the exclusive or, or XOR operator, so (P $\oplus$ Q) is true if exactly one of P and Q is true.

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
    "id": "nonlocal_games__chsh_discussion",
    "title": "Discussion: Probability of Victory for Quantum Strategy"
})

The above quantum strategy adopted by Alice and Bob offers a win rate of $\frac{2 + \sqrt{2}}{4}$, or about $85.36\%$. Let's see why this is the case.

First, consider the outcome if Alice and Bob simply measure their qubits in the Z basis without manipulating them at all. Because of the entanglement inherent to the Bell state they hold, their measurements will always agree (i.e., both true or both false).
This will suffice for victory in the three scenarios (0,0), (0,1) and (1,0) and fail for (1,1), so their win probability is $75\%$, the same as that for the straightforward classical strategies of invariably returning both true or both false.

Now let's analyze the optimal quantum strategy.

> As a reminder, probability "wavefunction" for a two-qubit state is given by the following length-4 vector of amplitudes:
> $$\begin{bmatrix}\psi_{00}\\\psi_{01}\\\psi_{10}\\\psi_{11}\end{bmatrix}$$
> $|\psi_{ij}|^2$ gives the probability of observing the corresponding basis state $\ket{ij}$ upon measuring the qubit pair.

The initial state $\ket{00}$ has $\psi_{00} = 1$ and $\psi_{01} = \psi_{10} = \psi_{11} = 0$.
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

The result of AND on their input bits will always be 0; thus they win when their outputs agree.  These two cases correspond to the top and bottom elements of the vectors above, with a combined probability of $(0.6533)^2 + (0.6533)^2 = 0.4268 + 0.4268 = 0.8536$, so they have an $85.36\%$ win chance.

## Case 2: Alice holds bit 1

When Alice holds bit 1, she measures in basis X (or, equivalently, Hadamard-transforms her qubit, leaving Bob's qubit unchanged,
before making her Z-basis measurement).  This corresponds to applying the operator
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

When Bob holds 0, they still want to return the same parity, which they again do with $85.36\%$ probability (left-hand vector above).
But when Bob holds 1, the AND condition is now true and the players want to answer in opposite parity. This corresponds to the second and third elements of the right-hand vector above.
Thanks to the "magic" of the combination of the counterclockwise rotation and Hadamard transform, they now do this with probability $(0.6533)^2 + (0.6533)^2 = 0.8536$ and thus $85.36\%$ becomes their win odds once more.

## Side notes

- If Bob never rotated his qubit, their entangled state would remain the Bell state if Alice held bit 0 and the state corresponding to $\frac12 \big(\ket{00} + \ket{01} + \ket{10} - \ket{11}\big)$ if Alice held bit 1.
While she and Bob would have a $100\%$ success probability against Alice's 0 bit, they would have only a $50\%$ chance of success if she held bit 1, and thus their win chance would revert to the $75\%$ of the classical strategy again.
- It can be proven that Alice and Bob cannot surpass an overall win probability of $85.36\%$ in the CHSH game. This entails a higher-level discussion of quantum observable theory, for instance see [Tsirelson's bound](https://en.wikipedia.org/wiki/Tsirelson's_bound).

@[section]({
    "id": "nonlocal_games__chsh_e2e",
    "title": "Running CHSH Game End to End"
})

Putting together the building blocks we've implemented into a strategy is very simple:
- Allocate two qubits and prepare a Bell state on them.
- Send one of the qubits to Alice and another to Bob (this step is "virtual", not directly reflected in Q# code, other than making sure that Alice and Bob each act on their qubit only).
- Have Alice and Bob perform their measurements on their respective qubits using `AliceQuantum` and `BobQuantum` operations.
- Reset used qubits to $\ket{0}$ before they are released.
- Return their measurement results.

In the example below you can compare winning percentage of classical and quantum games.

>You may play with the code and check if there is a difference in results when
>1. The referee picks non-random bits. How can the referee minimize Alice and Bob's win probability?
>2. Bob's qubit is measured first.
>3. Alice and Bob get unentangled qubit pair.

@[example]({"id": "nonlocal_games__chsh_e2edemo", "codePath": "./examples/CHSHGameDemo.qs"})

@[section]({
    "id": "nonlocal_games__ghz_game",
    "title": "GHZ Game"
})

In **GHZ Game** three players (Alice, Bob and Charlie) try to win the following game:

Each of them is given a bit (R, S and T respectively), and they have to return new bits (A, B and C respectively) so that 
R $\lor$ S $\lor$ T = A $\oplus$ B $\oplus$ C.
The input bits will have zero or two bits set to true and three or one bits set to false. 
The players are free to share information (and even qubits!) before the game starts, but are forbidden from communicating
with each other afterwards.

> - $\lor$ is the standard bitwise OR operator.
> - $\oplus$ is the exclusive or, or XOR operator, so (P $\oplus$ Q) is true if exactly one of P and Q is true.

To start with, take a look at how you would play the classical variant of this game without access to any quantum tools.
Then, let's proceed with quantum strategy and game implementation.

@[exercise]({
    "id": "nonlocal_games__ghz_win_condition",
    "title": "Win Condition",
    "path": "./ghz_win_condition/"
})

@[exercise]({
    "id": "nonlocal_games__ghz_classical_strategy",
    "title": "Classical Strategy",
    "path": "./ghz_classical_strategy/"
})

@[exercise]({
    "id": "nonlocal_games__ghz_classical_game",
    "title": "Classical GHZ Game",
    "path": "./ghz_classical_game/"
})

@[exercise]({
    "id": "nonlocal_games__ghz_create_ghz_state",
    "title": "Create Entangled Triple",
    "path": "./ghz_create_entangled_triple/"
})

@[exercise]({
    "id": "nonlocal_games__ghz_quantum_strategy",
    "title": "Quantum Strategy",
    "path": "./ghz_quantum_strategy/"
})

@[section]({
    "id": "nonlocal_games__ghz_discussion",
    "title": "Discussion: Why the GHZ quantum strategy has a 100% win rate"
})
---------------------------------------------------------------
Recall the formula for the win condition:
1. The sum of the answer bits must be even if the question bits are (0,0,0)
2. The sum of the answer bits must be odd if the question bits are (1,1,0), (1,0,1) or (0,1,1).

> As a reminder, the probability "wavefunction" for three qubits is given by the following vector of length 8:
> $$\begin{bmatrix}
\psi_{000}\\
\psi_{001}\\
\psi_{010}\\
\psi_{011}\\
\psi_{100}\\
\psi_{101}\\
\psi_{110}\\
\psi_{111}
\end{bmatrix}$$
> $|\psi_{ijk}|^2$ gives the probability of observing the corresponding basis state $\ket{ijk}$ upon measuring the qubit trio.

Now, the entangled state $\ket{\Phi}$ that Alice, Bob and Charlie have agreed to use is represented as

$$\begin{bmatrix}
+1/2\\
 0\\
 0\\
-1/2\\
 0\\
-1/2\\
-1/2\\
 0
\end{bmatrix}$$

Let's first consider the case in which **all three players got the 0 bit**.

When the players make their measurements, they will collectively get one of the basis states of the original state - 000, 011, 101 or 110.
This measn they'll report back zero \"1\" bits between them (with $25\%$ probability) or two \"1\" bits between them (with $75\%$ probability),
either way satisfying the win condition for the team.

Now, suppose **Alice gets a 0 bit and the others get 1**.

Alice, looking at the 0, takes a Z basis measurement as before, while Bob and Charlie each take X basis measurements.
(An X basis measurement is also equivalent to performing a Hadamard transform followed by a standard Z basis measurement,
as the X basis is the $\ket{+}$ / $\ket{-}$, and a Hadamard transform rotates the $\ket{0}$ / $\ket{1}$ basis to $\ket{+}$ / $\ket{-}$.)
So Bob and Charlie apply a Hadamard transform to their qubits, which corresponds to the following transformation applied to the whole system state:

$$I \otimes H \otimes H = \begin{bmatrix}
1/2 & 1/2 & 1/2 & 1/2 & 0 & 0 & 0 & 0\\
1/2 & -1/2 & 1/2 & -1/2 & 0 & 0 & 0 & 0\\
1/2 & 1/2 & -1/2 & -1/2 & 0 & 0 & 0 & 0\\
1/2 & -1/2 & -1/2 & 1/2 & 0 & 0 & 0 & 0\\
0 & 0 & 0 & 0 & 1/2 & 1/2 & 1/2 & 1/2\\
0 & 0 & 0 & 0 & 1/2 & -1/2 & 1/2 & -1/2\\
0 & 0 & 0 & 0 & 1/2 & 1/2 & -1/2 & -1/2\\
0 & 0 & 0 & 0 & 1/2 & -1/2 & -1/2 & 1/2
\end{bmatrix}$$

When applied to the original entangled state, all the amplitude shifts to the states corresponding to $\ket{001}$, $\ket{010}$, $\ket{100}$, and $\ket{111}$.
The precise configuration of the new entangled state is

$$\begin{bmatrix}
 0\\
 1/2\\
 1/2\\
 0\\
-1/2\\
 0\\
 0\\
 1/2
\end{bmatrix}$$

Now the players perform their measurements, and an odd number of them will see \"1\" (thanks to the new entangled state), again satisfying the win condition.
Similarly, if **Alice and Charlie get \"1\" bits and Bob a \"0\"**, Alice and Charlie will apply Hadamard transforms to their qubits to give the tensor product

$$ H \otimes I \otimes H = \begin{bmatrix}
1/2 & 1/2  & 0   & 0    & 1/2  & 1/2  & 0    & 0\\
1/2 & -1/2 & 0   & 0    & 1/2  & -1/2 & 0    & 0\\
0   & 0    & 1/2 & 1/2  & 0    & 0    & 1/2  & 1/2\\
0   & 0    & 1/2 & -1/2 & 0    & 0    & 1/2  & -1/2\\
1/2 & 1/2  & 0   & 0    & -1/2 & -1/2 & 0    & 0\\
1/2 & -1/2 & 0   & 0    & -1/2 & 1/2  & 0    & 0\\
0   & 0    & 1/2 & 1/2  & 0    & 0    & -1/2 & -1/2\\
0   & 0    & 1/2 & -1/2 & 0    & 0    & -1/2 & 1/2
\end{bmatrix}$$

The resulting state vector before the measurement will be the same as in the previous case, except that the $\ket{010}$ state
ends up with the negative amplitude instead of $\ket{100}$. Again the players report back an odd number of true bits between them and the team wins.

Finally if Charlie got the \"0\" bit and Alice and Bob both got \"1\", the latter two will apply Hadamard transform for the tensor product

$$ H \otimes H \otimes I = \begin{bmatrix}
1/2 & 0 & 1/2 & 0 & 1/2 & 0 & 1/2 & 0\\
0 & 1/2 & 0 & 1/2 & 0 & 1/2 & 0 & 1/2\\
1/2 & 0 & -1/2 & 0 & 1/2 & 0 & -1/2 & 0\\
0 & 1/2 & 0 & -1/2 & 0 & 1/2 & 0 & -1/2\\
1/2 & 0 & 1/2 & 0 & -1/2 & 0 & -1/2 & 0\\
0 & 1/2 & 0 & 1/2 & 0 & -1/2 & 0 & -1/2\\
1/2 & 0 & -1/2 & 0 & -1/2 & 0 & 1/2 & 0\\
0 & 1/2 & 0 & -1/2 & 0 & -1/2 & 0 & 1/2
\end{bmatrix}$$

Operating with this on the original entangled state yields $(\ket{100} + \ket{010} - \ket{001} + \ket{111})/2$ and 
once more the team will report back an odd number of true bits between them and win.

@[section]({
    "id": "nonlocal_games__ghz_e2e",
    "title": "Running GHZ Game End to End"
})

Putting together the building blocks we've implemented into a strategy is very simple:

1. Allocate three qubits and prepare our entangled state on them (using `CreateEntangledTriple`).
2. Send one of the qubits to each of the players (this step is \"virtual\", not directly reflected in Q# code, other than making sure that the strategies each act on their qubit only).
3. Have the players perform their measurements on their respective qubits using corresponding elements of the `strategies` array.
4. Reset qubits to $\ket{0}$ before they are released.
5. Return their measurement results.

In the example below you can compare winning percentage of classical and quantum games.

>You may play with the code and check if there is a difference in results when
>1. The referee picks non-random bits. How can the referee minimize player's win probability?
>2. Players get partially entangled qubit triple.

@[example]({"id": "nonlocal_games__ghz_e2edemo", "codePath": "./examples/GHZGameDemo.qs"})

@[section]({
    "id": "nonlocal_games__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned how to use quantum entanglement in nonlocal quantum games to get results that are better than any classical strategy can offer.
