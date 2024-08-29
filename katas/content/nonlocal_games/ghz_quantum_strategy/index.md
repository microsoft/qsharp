In this task you have to implement three functions, one for each player's quantum strategy.
Note that they are covered by one test, so you have to implement all of them to pass the test.

**Inputs:**

1. The input bit for one of each of the players (R, S and T respectively),
2. That player's qubit of the entangled triple shared between the players.

**Goal:**
Measure the qubit in the Z basis if the bit is 0 (FALSE), or the X basis if the bit is 1 (TRUE), and return the result.
The state of the qubit after the operation does not matter.
