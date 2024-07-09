# Nonlocal Games

@[section]({
    "id": "nonlocal_games__overview",
    "title": "Overview"
})

For our context, "nonlocal" means that the playing parties are separated by a great distance,
so they cannot communicate with each other during the game.
In a new kata, we discuss three quantum nonlocal games that display "quantum pseudo-telepathy" -
the use of quantum entanglement to eliminate the need for classical communication.
Another characteristics of these games is that they are "refereed", which means the players try to win against the referee.

**This kata covers the following topics:**
 - Clauser, Horne, Shimony, and Hold thought experiment (often abbreviated as CHSH game)
 - Greenberger-Horne-Zeilinger game (often abbreviated as GHZ game)
 - The Mermin-Peres Magic Square (entanglement) game

**What you should know to start working on this kata:**
 - Basic linear algebra
 - Single and multi-qubit systems
 - Single and multi-qubit gates
 - Single-qubit system measurements

@[section]({
    "id": "nonlocal_games__chsh_game",
    "title": "CHSH game"
})

The **CHSH Game** quantum kata is a series of exercises designed
to get you familiar with the CHSH game.

In it, two players (Alice and Bob) try to win the following game:

Each of them is given a bit (Alice gets X and Bob gets Y), and
they have to return new bits (Alice returns A and Bob returns B)
so that X ∧ Y = A ⊕ B. The trick is, they can not communicate during the game.

> * ∧ is the standard bitwise AND operator.
> * ⊕ is the exclusive or, or XOR operator, so (P ⊕ Q) is true if exactly one of P and Q is true.

* You can read more about CHSH game in the [lecture notes](https://cs.uwaterloo.ca/~watrous/QC-notes/QC-notes.20.pdf) by
  John Watrous.
* At the end of the section you can find an implementation of the CHSH game that includes an explanation of the history and theory behind the game. 

@[section]({
    "id": "nonlocal_games__chsh_game_classical",
    "title": "Part I. Classical CHSH"
})

@[exercise]({
    "id": "nonlocal_games__chsh_classical_win_condition",
    "title": "Win Condition",
    "path": "./chsh_classical_win_condition/"
})

@[exercise]({
    "id": "nonlocal_games__chsh_classical_strategy",
    "title": "Alice and Bob's classical strategy",
    "path": "./chsh_classical_strategy/"
})

@[section]({
    "id": "nonlocal_games__conclusion", 
    "title": "Conclusion" 
})

Congratulations! In this kata you learned how to communicate using quantum entanglement in nonlocal quantum games.