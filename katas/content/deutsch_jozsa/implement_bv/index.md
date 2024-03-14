**Inputs:** 
1. The number of bits in the input $N$ ($1 \le N \le 3$).
2. The "black box" oracle that takes an array of qubits as an argument and implements $f(x)$ as a phase oracle.  
  You are guaranteed that the function implemented by the oracle can be represented as a scalar product $f(x) = x \cdot s$ for some bit string $s$.

**Goal:** Return the bit string $s$.
You can use only one oracle call!
