**Input:**  A register of $N$ qubits in an arbitrary state.

**Goal:**  Flip the sign of the state of the register if it is *not* in the $\ket{0...0}$ state.
That is, if the register is in state $\ket{0...0}$, leave it unchanged,
but if it is in any other basis state, multiply its phase by $-1$.

> This operation implements operator 
> $$2\ket{0...0}\bra{0...0} - I = 
\begin{bmatrix}
  1 & 0 & ... & 0 \\
  0 &-1 & ... & 0 \\
  \vdots & \vdots & \ddots & \vdots \\ 
  0 & 0 & ... & -1
\end{bmatrix}$$
