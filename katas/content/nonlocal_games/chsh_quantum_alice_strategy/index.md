In the quantum version of the game, the players still can not communicate during the game,
but they are allowed to share qubits from a Bell pair before the start of the game.

**Inputs:**

- Alice's starting bit (X).
- Alice's half of Bell pair she shares with Bob.

**Goal:**
  Measure Alice's qubit in the Z basis if her bit is 0 (false), or the X basis if her bit is 1 (true) and
  return the measurement result as Boolean value: map `Zero` to false and `One` to true.
  The state of the qubit after the operation does not matter.
