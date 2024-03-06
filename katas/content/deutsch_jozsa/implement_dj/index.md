**Inputs:** 
1. The number of bits in the input $N$ ($1 \le N \le 3$).
2. The "black box" oracle that takes an array of qubits as an argument and implements $f(x)$ as a phase oracle.  
  You are guaranteed that the function implemented by the oracle is either constant or balanced.

**Goal:** Return `true` if the function is constant, or `false` if it is balanced.
You can use only one oracle call!
