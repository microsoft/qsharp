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

> * ∧ is the standard bitwise AND operator.
> * ⊕ is the exclusive or, or XOR operator, so (P ⊕ Q) is true if exactly one of P and Q is true.

@[section]({
    "id": "nonlocal_games__chsh_game_classical",
    "title": "Part I. Classical CHSH"
})

To start with, let's take a look at how you would play the classical variant of this game without access to any quantum tools.

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

@[section]({
    "id": "nonlocal_games__ghz_game",
    "title": "GHZ Game"
})

In **GHZ Game** three players (Alice, Bob and Charlie) try to win the following game:

Each of them is given a bit (r, s and t respectively), and they have to return new bits (a, b and c respectively) 
so that **r ∨ s ∨ t = a ⊕ b ⊕ c**.
The input bits will have zero or two bits set to true and three or one bits set to false. 
The players are free to share information (and even qubits!) before the game starts, but are forbidden from communicating
with each other afterwards.

- ∨ is the standard bitwise OR operator.
- ⊕ is the exclusive or, or XOR operator, so (P ⊕ Q) is true if exactly one of P and Q is true.

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

@[section]({
    "id": "nonlocal_games__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned how to use quantum entanglement in nonlocal quantum games to get results that are better than any classical strategy can offer.
