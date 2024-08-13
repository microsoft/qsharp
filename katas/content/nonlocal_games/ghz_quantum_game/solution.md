Putting together the building blocks we've implemented into a strategy is very simple:

1. Allocate three qubits and prepare our entangled state on them (using `CreateEntangledTriple`).
2. Send one of the qubits to each of the players (this step is \"virtual\", not directly reflected in Q# code, other than making sure that the strategies each act on their qubit only).
3. Have the players perform their measurements on their respective qubits using corresponding elements of the `strategies` array.
4. Return their measurement results.

@[solution]({
    "id": "nonlocal_games__ghz_quantum_game_solution",
    "codePath": "Solution.qs"
})
