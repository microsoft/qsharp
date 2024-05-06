**Inputs:** 

1. $N$ ($N \ge 1$) qubits in the $|0 \dots 0\rangle$ state.
2. A boolean `isEven`.

**Goal:**  Prepare a superposition of all *even* numbers if `isEven` is `true`, or of all *odd* numbers if `isEven` is `false`.  
A basis state encodes an integer number using [big-endian](https://en.wikipedia.org/wiki/Endianness) binary notation: state $|01\rangle$ corresponds to the integer $1$, and state $|10 \rangle$ - to the integer $2$.  

> For example, for $N = 2$ and `isEven = false` you need to prepare superposition $\frac{1}{\sqrt{2}} \big (|01\rangle + |11\rangle\big )$,  
and for $N = 2$ and `isEven = true` - superposition $\frac{1}{\sqrt{2}} \big (|00\rangle + |10\rangle\big )$.