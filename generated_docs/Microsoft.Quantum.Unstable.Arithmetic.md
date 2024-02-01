# Summary
This applies the in-place majority operation to 3 qubits.

# Description
Assuming the state of the input qubits are |x⟩, |y⟩ and |z⟩, then
this operation performs the following transformation:
|x⟩|y⟩|z⟩ ↦ |x ⊕ z⟩|y ⊕ z⟩MAJ(x, y, z).

# Input
## x
The first input qubit.
## y
The second input qubit.
## z
A qubit onto which the majority function will be applied.
---
operation MAJ(x : Qubit, y : Qubit, z : Qubit) : Unit is Adj + Ctl

---

# Summary
Reflects a quantum register about a given classical integer.

# Description
Given a quantum register initially in the state ∑ᵢ(αᵢ|i⟩),
where each |i⟩ is a basis state representing an integer i,
reflects the state of the register about the basis state |j⟩
for a given integer j: ∑ᵢ(-1)^(δᵢⱼ)(αᵢ|i⟩)

# Input
## index
The classical integer j indexing the basis state about which to reflect.
## reg
Little-endian quantum register to reflect.

# Remarks
This operation is implemented in-place, without explicit allocation of
additional auxiliary qubits.
---
operation ReflectAboutInteger(index : Int, reg : Qubit[]) : Unit is Adj + Ctl

---

# Summary
Increments a little-endian register ys by an integer number c

# Description
Computes ys += c modulo 2ⁿ, where ys is a little-endian register,
Length(ys) = n > 0, c is a Int number, 0 ≤ c < 2ⁿ.
NOTE: Use IncByIUsingIncByLE directly if the choice of implementation
is important.
---
operation IncByI(c : Int, ys : Qubit[]) : Unit is Adj + Ctl

---

# Summary
Increments a little-endian register ys by a BigInt number c

# Description
Computes ys += c modulo 2ⁿ, where ys is a little-endian register,
Length(ys) = n > 0, c is a BigInt number, 0 ≤ c < 2ⁿ.
NOTE: Use IncByLUsingIncByLE directly if the choice of implementation
is important.
---
operation IncByL(c : BigInt, ys : Qubit[]) : Unit is Adj + Ctl

---

# Summary
Increments a little-endian register ys by a little-endian register xs

# Description
Computes ys += xs modulo 2ⁿ, where xs and ys are little-endian registers,
and Length(xs) ≤ Length(ys) = n.
NOTE: Use operations like RippleCarryCGIncByLE directly if
the choice of implementation is important.
---
operation IncByLE(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl

---

# Summary
Sets a zero-initialized little-endian register zs to the sum of
little-endian registers xs and ys

# Description
Computes zs := xs + ys modulo 2ⁿ, where xs, ys, and zs are little-endian registers,
Length(xs) = Length(ys) ≤ Length(zs) = n, assuming zs is 0-initialized.
NOTE: Use operations like RippleCarryCGAddLE directly if
the choice of implementation is important.
---
operation AddLE(xs : Qubit[], ys : Qubit[], zs : Qubit[]) : Unit is Adj

---

# Summary
Reversible, in-place ripple-carry addition of two integers.

# Description
Computes ys += xs modulo 2ⁿ, where xs and ys are little-endian registers,
and Length(xs) ≤ Length(ys) = n.
This operation uses the ripple-carry algorithm.
Note that if Length(ys) >= Length(xs)+2, xs is padded with 0-initialized
qubits to match ys's length. The operation doesn't use any auxilliary
qubits otherwise.

# References
    - [arXiv:0910.2530](https://arxiv.org/abs/0910.2530)
      "Quantum Addition Circuits and Unbounded Fan-Out"
      by Yasuhiro Takahashi, Seiichiro Tani, Noboru Kunihiro

---
operation RippleCarryTTKIncByLE(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl

---

# Summary
Increments a little-endian register ys by a little-endian register xs
using the ripple-carry algorithm.

# Description
Computes ys += xs modulo 2ⁿ, where xs and ys are little-endian registers,
and Length(xs) ≤ Length(ys) = n.
Note that if Length(xs) != Length(ys), xs is padded with 0-initialized
qubits to match ys's length.
This operation uses the ripple-carry algorithm.

# Reference
    - [arXiv:1709.06648](https://arxiv.org/pdf/1709.06648.pdf)
      "Halving the cost of quantum addition" by Craig Gidney.
---
operation RippleCarryCGIncByLE(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl

---

# Summary
Sets a zero-initialized little-endian register zs to the sum of
little-endian registers xs and ys using the ripple-carry algorithm.

# Description
Computes zs := xs + ys + zs[0] modulo 2ⁿ, where xs, ys, and zs are
little-endian registers, Length(xs) = Length(ys) ≤ Length(zs) = n,
assuming zs is 0-initialized, except for maybe zs[0], which can be
This operation uses the ripple-carry algorithm.
NOTE: `zs[Length(xs)]` can be used as carry-out, if `zs` is longer than `xs`.

# Reference
    - [arXiv:1709.06648](https://arxiv.org/pdf/1709.06648.pdf)
      "Halving the cost of quantum addition" by Craig Gidney.
---
operation RippleCarryCGAddLE(xs : Qubit[], ys : Qubit[], zs : Qubit[]) : Unit is Adj

---

# Summary
Sets a zero-initialized little-endian register zs to the sum of
little-endian registers xs and ys using the carry-lookahead algorithm.

# Description
Computes zs := xs + ys + zs[0] modulo 2ⁿ, where xs, ys, and zs are
little-endian registers, Length(xs) = Length(ys) ≤ Length(zs) = n,
assuming zs is 0-initialized, except for maybe zs[0], which can be
in |0> or |1> state and can be used as carry-in.
NOTE: `zs[Length(xs)]` can be used as carry-out, if `zs` is longer than `xs`.
This operation uses the carry-lookahead algorithm.

# Reference
    - [arXiv:quant-ph/0406142](https://arxiv.org/abs/quant-ph/0406142)
     "A logarithmic-depth quantum carry-lookahead adder" by
     Thomas G. Draper, Samuel A. Kutin, Eric M. Rains, Krysta M. Svore
---
operation LookAheadDKRSAddLE(xs : Qubit[], ys : Qubit[], zs : Qubit[]) : Unit is Adj

---

# Summary
Increments a little-endian register ys by a little-endian register xs
using Quantum Fourier Transform.

# Description
Computes ys += xs modulo 2ⁿ, where xs and ys are little-endian registers,
and Length(xs) = Length(ys) = n.
This operation uses Quantum Fourier Transform.

# Reference
    - [arXiv:quant-ph/0008033](https://arxiv.org/abs/quant-ph/0008033)
     "Addition on a Quantum Computer" by Thomas G. Draper
---
operation FourierTDIncByLE(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl

---

# Summary
Increments a little-endian register ys by a BigInt number c
using provided adder.

# Description
Computes ys += c modulo 2ⁿ, where ys is a little-endian register
Length(ys) = n > 0, c is a BigInt number, 0 ≤ c < 2ⁿ.
---
operation IncByLUsingIncByLE(adder : ((Qubit[], Qubit[]) => Unit is Param<0>), c : BigInt, ys : Qubit[]) : Unit is Adj + Ctl

---

# Summary
Increments a little-endian register ys by an Int number c
using provided adder.

# Description
Computes ys += c modulo 2ⁿ, where ys is a little-endian register
Length(ys) = n > 0, c is an Int number, 0 ≤ c < 2ⁿ.
---
operation IncByIUsingIncByLE(adder : ((Qubit[], Qubit[]) => Unit is Param<0>), c : Int, ys : Qubit[]) : Unit is Adj + Ctl

---

# Summary
Generic operation to turn two out-place adders into one in-place adder

# Description
This implementation allows to specify two distinct adders for forward
and backward direction.  The forward adder is always applied in its
body variant, whereas the backward adder is always applied in its adjoint
variant.  Therefore, it's possible to, for example, use the ripple-carry
out-of-place adder in backwards direction to require no T gates.

The controlled variant is also optimized in a way that everything but
the adders is controlled,

# Reference
    - [arXiv:2012.01624](https://arxiv.org/abs/2012.01624)
      "Quantum block lookahead adders and the wait for magic states"
      by by Craig Gidney.
---
operation IncByLEUsingAddLE(forwardAdder : ((Qubit[], Qubit[], Qubit[]) => Unit is Param<0>), backwardAdder : ((Qubit[], Qubit[], Qubit[]) => Unit is Param<1>), xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl

---

# Summary
Computes `if (c < x) { action(target) }`, that is, applies `action` to `target`
if a BigInt value `c` is less than the little-endian qubit register `x`
---
operation ApplyIfLessL<'T>(action : ('T => Unit is Param<1>), c : BigInt, x : Qubit[], target : 'T) : Unit is Adj + Ctl

---

# Summary
Computes `if (c <= x) { action(target) }`, that is, applies `action` to `target`
if a BigInt value `c` is less or equal to the little-endian qubit register `x`
---
operation ApplyIfLessOrEqualL<'T>(action : ('T => Unit is Param<1>), c : BigInt, x : Qubit[], target : 'T) : Unit is Adj + Ctl

---

# Summary
Computes `if (c == x) { action(target) }`, that is, applies `action` to `target`
if a BigInt value `c` is equal to the little-endian qubit register `x`
---
operation ApplyIfEqualL<'T>(action : ('T => Unit is Param<1>), c : BigInt, xs : Qubit[], target : 'T) : Unit is Adj + Ctl

---

# Summary
Computes `if (c >= x) { action(target) }`, that is, applies `action` to `target`
if a BigInt value `c` is greater or equal to the little-endian qubit register `x`
---
operation ApplyIfGreaterOrEqualL<'T>(action : ('T => Unit is Param<1>), c : BigInt, x : Qubit[], target : 'T) : Unit is Adj + Ctl

---

# Summary
Computes `if (c > x) { action(target) }`, that is, applies `action` to `target`
if a BigInt value `c` is greater than the little-endian qubit register `x`
---
operation ApplyIfGreaterL<'T>(action : ('T => Unit is Param<1>), c : BigInt, x : Qubit[], target : 'T) : Unit is Adj + Ctl

---

# Summary
Computes `if x < y { action(target) }`, that is, applies `action` to `target`
if register `x` is less than the register `y`.
Both qubit registers should be in a little-endian format.
---
operation ApplyIfLessLE<'T>(action : ('T => Unit is Param<1>), x : Qubit[], y : Qubit[], target : 'T) : Unit is Adj + Ctl

---

# Summary
Computes `if x <= y { action(target) }`, that is, applies `action` to `target`
if register `x` is less or equal to the register `y`.
Both qubit registers should be in a little-endian format.
---
operation ApplyIfLessOrEqualLE<'T>(action : ('T => Unit is Param<1>), x : Qubit[], y : Qubit[], target : 'T) : Unit is Adj + Ctl

---

# Summary
Computes `if x == y { action(target) }`, that is, applies `action` to `target`
if register `x` is equal to the register `y`.
Both qubit registers should be in a little-endian format.
---
operation ApplyIfEqualLE<'T>(action : ('T => Unit is Param<1>), x : Qubit[], y : Qubit[], target : 'T) : Unit is Adj + Ctl

---

# Summary
Computes `if x >= y { action(target) }`, that is, applies `action` to `target`
if register `x` is greater or equal to the register `y`.
Both qubit registers should be in a little-endian format.
---
operation ApplyIfGreaterOrEqualLE<'T>(action : ('T => Unit is Param<1>), x : Qubit[], y : Qubit[], target : 'T) : Unit is Adj + Ctl

---

# Summary
Computes `if x > y { action(target) }`, that is, applies `action` to `target`
if register `x` is greater than the register `y`.
Both qubit registers should be in a little-endian format.
---
operation ApplyIfGreaterLE<'T>(action : ('T => Unit is Param<1>), x : Qubit[], y : Qubit[], target : 'T) : Unit is Adj + Ctl
