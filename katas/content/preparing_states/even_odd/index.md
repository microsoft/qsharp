**Inputs:** 

1. $N$ ($N \ge 1$) qubits in the $\ket{0 \dots 0}$ state.
2. A boolean `isEven`.

**Goal:**  Prepare a superposition of all *even* numbers if `isEven` is `true`, or of all *odd* numbers if `isEven` is `false`.  
A basis state encodes an integer number using [big-endian](https://en.wikipedia.org/wiki/Endianness) binary notation: state $\ket{01}$ corresponds to the integer $1$, and state $\ket{10 }$ - to the integer $2$.  

> For example, for $N = 2$ and `isEven = false` you need to prepare superposition $\frac{1}{\sqrt{2}} \big (\ket{01} + \ket{11}\big )$,  
and for $N = 2$ and `isEven = true` - superposition $\frac{1}{\sqrt{2}} \big (\ket{00} + \ket{10}\big )$.