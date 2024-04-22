**Inputs:**

1. $N$ qubits in the $|0 \dots 0\rangle$ state.
2. Four bit strings represented as `Bool[][]` `bits`.

    `bits` is an array of size $4 \times N$ which describes the bit strings as follows:
    - `bits[i]` describes the *i*th bit string and has $N$ elements.
    - All four bit strings will be distinct.

**Goal:** Create an equal superposition of the four basis states given by the bit strings.

**Example:**

For $N = 3$ and `bits =  [[false, true, false], [true, false, false], [false, false, true], [true, true, false]]`, the state you need to prepare is $\frac{1}{2} \big(|010\rangle + |100\rangle + |001\rangle + |110\rangle\big)$.
