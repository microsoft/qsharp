// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Unstable.Arithmetic {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;

    /// # Summary
    /// This applies the in-place majority operation to 3 qubits.
    ///
    /// # Description
    /// Assuming the state of the input qubits are |x⟩, |y⟩ and |z⟩, then
    /// this operation performs the following transformation:
    /// |x⟩|y⟩|z⟩ ↦ |x ⊕ z⟩|y ⊕ z⟩MAJ(x, y, z).
    ///
    /// # Input
    /// ## x
    /// The first input qubit.
    /// ## y
    /// The second input qubit.
    /// ## z
    /// A qubit onto which the majority function will be applied.
    operation MAJ(x : Qubit, y : Qubit, z : Qubit) : Unit is Adj + Ctl {
        CNOT(z, y);
        CNOT(z, x);
        CCNOT(y, x, z);
    }

    /// # Summary
    /// Reflects a quantum register about a given classical integer.
    ///
    /// # Description
    /// Given a quantum register initially in the state ∑ᵢ(αᵢ|i⟩),
    /// where each |i⟩ is a basis state representing an integer i,
    /// reflects the state of the register about the basis state |j⟩
    /// for a given integer j: ∑ᵢ(-1)^(δᵢⱼ)(αᵢ|i⟩)
    /// This operation is implemented in-place, without explicit allocation of
    /// additional auxiliary qubits.
    ///
    /// # Input
    /// ## index
    /// The classical integer j indexing the basis state about which to reflect.
    /// ## reg
    /// Little-endian quantum register to reflect.
    operation ReflectAboutInteger(index : Int, reg : Qubit[]) : Unit is Adj + Ctl {
        within {
            // Evaluation optimization for case index == 0
            if index == 0 {
                ApplyToEachA(X, reg);
            } else {
                // We want to reduce to the problem of reflecting about the all-ones
                // state. To do that, we apply our reflection within an application
                // of X instructions that flip all the zeros in our index.
                ApplyPauliFromInt(PauliX, false, index, reg);
            }
        } apply {
            Controlled ApplyAsSinglyControlled(Most(reg), (Z, Tail(reg)));
        }
    }

    //
    // Add, Increment      |   Operation    | Description
    // ____________________|________________|_______________________________________________________________
    // y += 5              | IncByI, IncByL | Increment LE register in-place by integer
    // y += x              | IncByLE        | Increment LE register in-place by LE register
    // z = x + 5 (z was 0) |                | Add integer to LE register creating result out-of-place
    // z = x + y (z was 0) | AddLE          | Add two LE register creating result out-of-place
    // z += x + 5          |                | Increment LE register by the sum of integer and LE register
    // z += x + y          |                | Increment LE register by the sum of two LE registers
    //
    // IncByLE implementations:
    //     RippleCarryTTKIncByLE (default)
    //     RippleCarryCGIncByLE
    //     FourierTDIncByLE
    //     via IncByLEUsingAddLE and any out-of-place addition
    // IncByI implementations:
    //     via IncByIUsingIncByLE and any in-place LE adder
    // IncByL implementations:
    //     via IncByLUsingIncByLE and any in-place LE adder
    // AddLE implementations:
    //     RippleCarryCGAddLE (default)
    //     LookAheadDKRSAddLE
    //

    /// # Summary
    /// Increments a little-endian register ys by an integer number c
    ///
    /// # Description
    /// Computes ys += c modulo 2ⁿ, where ys is a little-endian register,
    /// Length(ys) = n > 0, c is a Int number, 0 ≤ c < 2ⁿ.
    /// NOTE: Use IncByIUsingIncByLE directly if the choice of implementation
    /// is important.
    operation IncByI(c : Int, ys : Qubit[]) : Unit is Adj + Ctl {
        IncByIUsingIncByLE(RippleCarryTTKIncByLE, c, ys);
    }

    /// # Summary
    /// Increments a little-endian register ys by a BigInt number c
    ///
    /// # Description
    /// Computes ys += c modulo 2ⁿ, where ys is a little-endian register,
    /// Length(ys) = n > 0, c is a BigInt number, 0 ≤ c < 2ⁿ.
    /// NOTE: Use IncByLUsingIncByLE directly if the choice of implementation
    /// is important.
    operation IncByL(c : BigInt, ys : Qubit[]) : Unit is Adj + Ctl {
        IncByLUsingIncByLE(RippleCarryTTKIncByLE, c, ys);
    }

    /// # Summary
    /// Increments a little-endian register ys by a little-endian register xs
    ///
    /// # Description
    /// Computes ys += xs modulo 2ⁿ, where xs and ys are little-endian registers,
    /// and Length(xs) ≤ Length(ys) = n.
    /// NOTE: Use operations like RippleCarryCGIncByLE directly if
    /// the choice of implementation is important.
    operation IncByLE(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
        RippleCarryTTKIncByLE(xs, ys);
    }

    /// # Summary
    /// Sets a zero-initialized little-endian register zs to the sum of
    /// little-endian registers xs and ys
    ///
    /// # Description
    /// Computes zs := xs + ys modulo 2ⁿ, where xs, ys, and zs are little-endian registers,
    /// Length(xs) = Length(ys) ≤ Length(zs) = n, assuming zs is 0-initialized.
    /// NOTE: Use operations like RippleCarryCGAddLE directly if
    /// the choice of implementation is important.
    operation AddLE(xs : Qubit[], ys : Qubit[], zs : Qubit[]) : Unit is Adj {
        RippleCarryCGAddLE(xs, ys, zs);
    }

    /// # Summary
    /// Reversible, in-place ripple-carry addition of two integers.
    ///
    /// # Description
    /// Computes ys += xs modulo 2ⁿ, where xs and ys are little-endian registers,
    /// and Length(xs) ≤ Length(ys) = n.
    /// This operation uses the ripple-carry algorithm.
    /// Note that if Length(ys) >= Length(xs)+2, xs is padded with 0-initialized
    /// qubits to match ys's length. The operation doesn't use any auxiliary
    /// qubits otherwise.
    ///
    /// # References
    /// - [arXiv:0910.2530](https://arxiv.org/abs/0910.2530)
    ///   "Quantum Addition Circuits and Unbounded Fan-Out",
    ///   Yasuhiro Takahashi, Seiichiro Tani, Noboru Kunihiro
    operation RippleCarryTTKIncByLE(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
        let xsLen = Length(xs);
        let ysLen = Length(ys);

        Fact(ysLen >= xsLen, "Register `ys` must be longer than register `xs`.");
        Fact(xsLen >= 1, "Registers `xs` and `ys` must contain at least one qubit.");

        if xsLen == ysLen {
            if xsLen > 1 {
                within {
                    ApplyOuterTTKAdder(xs, ys);
                } apply {
                    ApplyInnerTTKAdderNoCarry(xs, ys);
                }
            }
            CNOT(xs[0], ys[0]);
        } elif xsLen + 1 == ysLen {
            if xsLen > 1 {
                CNOT(xs[xsLen - 1], ys[ysLen - 1]);
                within {
                    ApplyOuterTTKAdder(xs, ys);
                } apply {
                    ApplyInnerTTKAdderWithCarry(xs, ys);
                }
            } else {
                CCNOT(xs[0], ys[0], ys[1]);
            }
            CNOT(xs[0], ys[0]);
        } elif xsLen + 2 <= ysLen {
            // Pad xs so that its length is one qubit shorter than ys.
            use padding = Qubit[ysLen - xsLen - 1];
            RippleCarryTTKIncByLE(xs + padding, ys);
        }
    }

    /// # Summary
    /// Increments a little-endian register ys by a little-endian register xs
    /// using the ripple-carry algorithm.
    ///
    /// # Description
    /// Computes ys += xs modulo 2ⁿ, where xs and ys are little-endian registers,
    /// and Length(xs) ≤ Length(ys) = n.
    /// Note that if Length(xs) != Length(ys), xs is padded with 0-initialized
    /// qubits to match ys's length.
    /// This operation uses the ripple-carry algorithm.
    ///
    /// # Reference
    /// - [arXiv:1709.06648](https://arxiv.org/pdf/1709.06648.pdf)
    ///   "Halving the cost of quantum addition", Craig Gidney.
    operation RippleCarryCGIncByLE(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
        let xsLen = Length(xs);
        let ysLen = Length(ys);

        Fact(ysLen >= xsLen, "Register `ys` must be longer than register `xs`.");
        Fact(xsLen >= 1, "Registers `xs` and `ys` must contain at least one qubit.");

        if ysLen - xsLen >= 2 {
            // Pad xs so that its length is one qubit shorter than ys.
            use padding = Qubit[ysLen - xsLen - 1];
            RippleCarryCGIncByLE(xs + padding, ys);
        } elif xsLen == 1 {
            if ysLen == 1 {
                CNOT(xs[0], ys[0]);
            } elif ysLen == 2 {
                HalfAdderForInc(xs[0], ys[0], ys[1]);
            }
        } else {
            use carries = Qubit[xsLen];
            within {
                ApplyAndAssuming0Target(xs[0], ys[0], carries[0]);
            } apply {
                for i in 1..xsLen - 2 {
                    CarryForInc(carries[i - 1], xs[i], ys[i], carries[i]);
                }
                if xsLen == ysLen {
                    within {
                        CNOT(carries[xsLen - 2], xs[xsLen - 1]);
                    } apply {
                        CNOT(xs[xsLen - 1], ys[xsLen - 1]);
                    }
                } else {
                    FullAdderForInc(carries[xsLen - 2], xs[xsLen - 1], ys[xsLen - 1], ys[xsLen]);
                }
                for i in xsLen - 2..-1..1 {
                    UncarryForInc(carries[i - 1], xs[i], ys[i], carries[i]);
                }
            }
            CNOT(xs[0], ys[0]);
        }
    }

    /// # Summary
    /// Sets a zero-initialized little-endian register zs to the sum of
    /// little-endian registers xs and ys using the ripple-carry algorithm.
    ///
    /// # Description
    /// Computes zs := xs + ys + zs[0] modulo 2ⁿ, where xs, ys, and zs are
    /// little-endian registers, Length(xs) = Length(ys) ≤ Length(zs) = n,
    /// assuming zs is 0-initialized, except for maybe zs[0], which can be
    // in |0> or |1> state and can be used as carry-in.
    /// This operation uses the ripple-carry algorithm.
    /// NOTE: `zs[Length(xs)]` can be used as carry-out, if `zs` is longer than `xs`.
    ///
    /// # Reference
    /// - [arXiv:1709.06648](https://arxiv.org/pdf/1709.06648.pdf)
    ///   "Halving the cost of quantum addition", Craig Gidney.
    operation RippleCarryCGAddLE(xs : Qubit[], ys : Qubit[], zs : Qubit[]) : Unit is Adj {
        let xsLen = Length(xs);
        let zsLen = Length(zs);
        Fact(Length(ys) == xsLen, "Registers `xs` and `ys` must be of same length.");
        Fact(zsLen >= xsLen, "Register `zs` must be no shorter than register `xs`.");

        // Since zs is zero-initialized, its bits at indexes higher than
        // xsLen remain unused as there will be no carry into them.
        let top = MinI(zsLen - 2, xsLen - 1);
        for k in 0..top {
            FullAdder(zs[k], xs[k], ys[k], zs[k + 1]);
        }

        if xsLen > 0 and xsLen == zsLen {
            CNOT(Tail(xs), Tail(zs));
            CNOT(Tail(ys), Tail(zs));
        }
    }

    /// # Summary
    /// Sets a zero-initialized little-endian register zs to the sum of
    /// little-endian registers xs and ys using the carry-lookahead algorithm.
    ///
    /// # Description
    /// Computes zs := xs + ys + zs[0] modulo 2ⁿ, where xs, ys, and zs are
    /// little-endian registers, Length(xs) = Length(ys) ≤ Length(zs) = n,
    /// assuming zs is 0-initialized, except for maybe zs[0], which can be
    /// in |0> or |1> state and can be used as carry-in.
    /// NOTE: `zs[Length(xs)]` can be used as carry-out, if `zs` is longer than `xs`.
    /// This operation uses the carry-lookahead algorithm.
    ///
    /// # Reference
    /// - [arXiv:quant-ph/0406142](https://arxiv.org/abs/quant-ph/0406142)
    ///   "A logarithmic-depth quantum carry-lookahead adder",
    ///   Thomas G. Draper, Samuel A. Kutin, Eric M. Rains, Krysta M. Svore
    operation LookAheadDKRSAddLE(xs : Qubit[], ys : Qubit[], zs : Qubit[]) : Unit is Adj {
        let xsLen = Length(xs);
        let zsLen = Length(zs);
        Fact(Length(ys) == xsLen, "Registers `xs` and `ys` must be of same length.");
        Fact(zsLen >= xsLen, "Register `zs` must be no shorter than register `xs`.");

        if zsLen > xsLen {
            // with carry-out
            // compute initial generate values
            for k in 0..xsLen - 1 {
                ApplyAndAssuming0Target(xs[k], ys[k], zs[k + 1]);
            }

            within {
                // compute initial propagate values
                for i in IndexRange(xs) {
                    CNOT(xs[i], ys[i]);
                }
            } apply {
                if xsLen > 1 {
                    ComputeCarries(Rest(ys), zs[1..xsLen]);
                }

                // compute sum into carries
                for k in 0..xsLen - 1 {
                    CNOT(ys[k], zs[k]);
                }
            }
        } else {
            // xsLen == zsLen, so without carry-out
            LookAheadDKRSAddLE(Most(xs), Most(ys), zs);
            CNOT(Tail(xs), Tail(zs));
            CNOT(Tail(ys), Tail(zs));
        }
    }

    /// # Summary
    /// Increments a little-endian register ys by a little-endian register xs
    /// using Quantum Fourier Transform.
    ///
    /// # Description
    /// Computes ys += xs modulo 2ⁿ, where xs and ys are little-endian registers,
    /// and Length(xs) = Length(ys) = n.
    /// This operation uses Quantum Fourier Transform.
    ///
    /// # Reference
    /// - [arXiv:quant-ph/0008033](https://arxiv.org/abs/quant-ph/0008033)
    ///   "Addition on a Quantum Computer", Thomas G. Draper
    operation FourierTDIncByLE(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
        within {
            ApplyQFT(ys);
        } apply {
            for i in IndexRange(xs) {
                Controlled PhaseGradient([xs[i]], ys[i...]);
            }
        }
    }

    /// # Summary
    /// Increments a little-endian register ys by a BigInt number c
    /// using provided adder.
    ///
    /// # Description
    /// Computes ys += c modulo 2ⁿ, where ys is a little-endian register
    /// Length(ys) = n > 0, c is a BigInt number, 0 ≤ c < 2ⁿ.
    operation IncByLUsingIncByLE(
        adder : (Qubit[], Qubit[]) => Unit is Adj + Ctl,
        c : BigInt,
        ys : Qubit[]
    ) : Unit is Adj + Ctl {

        let ysLen = Length(ys);
        Fact(ysLen > 0, "Length of `ys` must be at least 1.");
        Fact(c >= 0L, "Constant `c` must be non-negative.");
        Fact(c < 2L^ysLen, "Constant `c` must be smaller than 2^Length(ys).");

        if c != 0L {
            // If c has j trailing zeros, then the j least significant
            // bits of y won't be affected by the addition and can
            // therefore be ignored by applying the addition only to
            // the other qubits and shifting c accordingly.
            let j = TrailingZeroCountL(c);
            use x = Qubit[ysLen - j];
            within {
                ApplyXorInPlaceL(c >>> j, x);
            } apply {
                adder(x, ys[j...]);
            }
        }
    }

    /// # Summary
    /// Increments a little-endian register ys by an Int number c
    /// using provided adder.
    ///
    /// # Description
    /// Computes ys += c modulo 2ⁿ, where ys is a little-endian register
    /// Length(ys) = n > 0, c is an Int number, 0 ≤ c < 2ⁿ.
    operation IncByIUsingIncByLE(
        adder : (Qubit[], Qubit[]) => Unit is Adj + Ctl,
        c : Int,
        ys : Qubit[]
    ) : Unit is Adj + Ctl {

        let ysLen = Length(ys);
        Fact(ysLen > 0, "Length of `ys` must be at least 1.");
        Fact(c >= 0, "Constant `c` must be non-negative.");
        Fact(c < 2^ysLen, "Constant `c` must be smaller than 2^Length(ys).");

        if c != 0 {
            // If c has j trailing zeros than the j least significant
            // bits of y won't be affected by the addition and can
            // therefore be ignored by applying the addition only to
            // the other qubits and shifting c accordingly.
            let j = TrailingZeroCountI(c);
            use x = Qubit[ysLen - j];
            within {
                ApplyXorInPlace(c >>> j, x);
            } apply {
                adder(x, ys[j...]);
            }
        }
    }

    /// # Summary
    /// Generic operation to turn two out-place adders into one in-place adder
    ///
    /// # Description
    /// This implementation allows to specify two distinct adders for forward
    /// and backward direction.  The forward adder is always applied in its
    /// body variant, whereas the backward adder is always applied in its adjoint
    /// variant.  Therefore, it's possible to, for example, use the ripple-carry
    /// out-of-place adder in backwards direction to require no T gates.
    ///
    /// The controlled variant is also optimized in a way that everything but
    /// the adders is controlled,
    ///
    /// # Reference
    /// - [arXiv:2012.01624](https://arxiv.org/abs/2012.01624)
    ///   "Quantum block lookahead adders and the wait for magic states",
    ///   Craig Gidney.
    operation IncByLEUsingAddLE(
        forwardAdder : (Qubit[], Qubit[], Qubit[]) => Unit is Adj,
        backwardAdder : (Qubit[], Qubit[], Qubit[]) => Unit is Adj,
        xs : Qubit[],
        ys : Qubit[]
    ) : Unit is Adj + Ctl {

        body (...) {
            let n = Length(xs);

            Fact(Length(ys) == n, "Registers xs and ys must be of same length");

            use qs = Qubit[n];

            forwardAdder(xs, ys, qs);
            for i in IndexRange(ys) {
                SWAP(ys[i], qs[i]);
            }
            ApplyToEachA(X, qs);
            within {
                ApplyToEachA(X, ys);
            } apply {
                Adjoint backwardAdder(xs, ys, qs);
            }
        }
        adjoint (...) {
            let n = Length(xs);

            Fact(Length(ys) == n, "Registers xs and ys must be of same length");

            use qs = Qubit[n];

            within {
                ApplyToEachA(X, ys);
            } apply {
                forwardAdder(xs, ys, qs);
            }
            ApplyToEachA(X, qs);
            for i in IndexRange(ys) {
                SWAP(ys[i], qs[i]);
            }
            Adjoint backwardAdder(xs, ys, qs);
        }
        controlled (ctls, ...) {
            // When we control everything except the adders, the adders will
            // cancel themselves.
            let n = Length(xs);

            Fact(Length(ys) == n, "Registers xs and ys must be of same length");

            use qs = Qubit[n];

            forwardAdder(xs, ys, qs);
            for i in IndexRange(ys) {
                Controlled SWAP(ctls, (ys[i], qs[i]))
            }
            ApplyToEachA(tgt => Controlled X(ctls, tgt), qs);
            within {
                ApplyToEachA(tgt => Controlled X(ctls, tgt), ys);
            } apply {
                Adjoint backwardAdder(xs, ys, qs);
            }
        }
        controlled adjoint (ctls, ...) {
            // When we control everything except the adders, the adders will
            // cancel themselves.
            let n = Length(xs);

            Fact(Length(ys) == n, "Registers xs and ys must be of same length");

            use qs = Qubit[n];

            within {
                ApplyToEachA(tgt => Controlled X(ctls, tgt), ys);
            } apply {
                forwardAdder(xs, ys, qs);
            }
            ApplyToEachA(tgt => Controlled X(ctls, tgt), qs);
            for i in IndexRange(ys) {
                Controlled SWAP(ctls, (ys[i], qs[i]))
            }
            Adjoint backwardAdder(xs, ys, qs);
        }
    }

    //
    // Comparisons
    //
    // Compare BigInt and qubit register in a little-endian format and apply action
    //   if c < x { action(target) }  | ApplyIfLessL
    //   if c <= x { action(target) } | ApplyIfLessOrEqualL
    //   if c == x { action(target) } | ApplyIfEqualL
    //   if c >= x { action(target) } | ApplyIfGreaterOrEqualL
    //   if c > x { action(target) }  | ApplyIfGreaterL
    //
    // Compare two qubit registers in a little-endian format and apply action
    //   if x < y { action(target) }  | ApplyIfLessLE
    //   if x <= y { action(target) } | ApplyIfLessOrEqualLE
    //   if x == y { action(target) } | ApplyIfEqualLE
    //   if x >= y { action(target) } | ApplyIfGreaterOrEqualLE
    //   if x > y { action(target) }  | ApplyIfGreaterLE
    //

    /// # Summary
    /// Computes `if (c < x) { action(target) }`, that is, applies `action` to `target`
    /// if a BigInt value `c` is less than the little-endian qubit register `x`
    operation ApplyIfLessL<'T>(
        action : 'T => Unit is Adj + Ctl,
        c : BigInt,
        x : Qubit[],
        target : 'T
    ) : Unit is Adj + Ctl {

        ApplyActionIfGreaterThanOrEqualConstant(false, action, c + 1L, x, target);
    }

    /// # Summary
    /// Computes `if (c <= x) { action(target) }`, that is, applies `action` to `target`
    /// if a BigInt value `c` is less or equal to the little-endian qubit register `x`
    operation ApplyIfLessOrEqualL<'T>(
        action : 'T => Unit is Adj + Ctl,
        c : BigInt,
        x : Qubit[],
        target : 'T
    ) : Unit is Adj + Ctl {

        ApplyActionIfGreaterThanOrEqualConstant(false, action, c, x, target);
    }

    /// # Summary
    /// Computes `if (c == x) { action(target) }`, that is, applies `action` to `target`
    /// if a BigInt value `c` is equal to the little-endian qubit register `x`
    operation ApplyIfEqualL<'T>(
        action : 'T => Unit is Adj + Ctl,
        c : BigInt,
        xs : Qubit[],
        target : 'T
    ) : Unit is Adj + Ctl {

        let cBitSize = BitSizeL(c);
        let xLen = Length(xs);
        if (cBitSize <= xLen) {
            let bits = BigIntAsBoolArray(c, Length(xs));
            within {
                ApplyPauliFromBitString(PauliX, false, bits, xs);
            } apply {
                Controlled ApplyAsSinglyControlled(xs, (a => action(a), target));
            }
        }
    }

    /// # Summary
    /// Computes `if (c >= x) { action(target) }`, that is, applies `action` to `target`
    /// if a BigInt value `c` is greater or equal to the little-endian qubit register `x`
    operation ApplyIfGreaterOrEqualL<'T>(
        action : 'T => Unit is Adj + Ctl,
        c : BigInt,
        x : Qubit[],
        target : 'T
    ) : Unit is Adj + Ctl {

        ApplyActionIfGreaterThanOrEqualConstant(true, action, c + 1L, x, target);
    }

    /// # Summary
    /// Computes `if (c > x) { action(target) }`, that is, applies `action` to `target`
    /// if a BigInt value `c` is greater than the little-endian qubit register `x`
    operation ApplyIfGreaterL<'T>(
        action : 'T => Unit is Adj + Ctl,
        c : BigInt,
        x : Qubit[],
        target : 'T
    ) : Unit is Adj + Ctl {

        ApplyActionIfGreaterThanOrEqualConstant(true, action, c, x, target);
    }

    /// # Summary
    /// Computes `if x < y { action(target) }`, that is, applies `action` to `target`
    /// if register `x` is less than the register `y`.
    /// Both qubit registers should be in a little-endian format.
    operation ApplyIfLessLE<'T>(
        action : 'T => Unit is Adj + Ctl,
        x : Qubit[],
        y : Qubit[],
        target : 'T
    ) : Unit is Adj + Ctl {

        ApplyIfGreaterLE(action, y, x, target);
    }

    /// # Summary
    /// Computes `if x <= y { action(target) }`, that is, applies `action` to `target`
    /// if register `x` is less or equal to the register `y`.
    /// Both qubit registers should be in a little-endian format.
    operation ApplyIfLessOrEqualLE<'T>(
        action : 'T => Unit is Adj + Ctl,
        x : Qubit[],
        y : Qubit[],
        target : 'T
    ) : Unit is Adj + Ctl {

        Fact(Length(x) > 0, "Bitwidth must be at least 1");
        within {
            ApplyToEachA(X, x);
        } apply {
            // control is not inverted
            ApplyActionIfSumOverflows(action, x, y, false, target);
        }
    }

    /// # Summary
    /// Computes `if x == y { action(target) }`, that is, applies `action` to `target`
    /// if register `x` is equal to the register `y`.
    /// Both qubit registers should be in a little-endian format.
    operation ApplyIfEqualLE<'T>(
        action : 'T => Unit is Adj + Ctl,
        x : Qubit[],
        y : Qubit[],
        target : 'T
    ) : Unit is Adj + Ctl {

        Fact(Length(x) == Length(y), "x and y must be of same length");
        within {
            for i in IndexRange(x) {
                CNOT(x[i], y[i]);
                X(y[i]);
            }
        } apply {
            Controlled ApplyAsSinglyControlled(y, (a => action(a), target))
        }
    }

    /// # Summary
    /// Computes `if x >= y { action(target) }`, that is, applies `action` to `target`
    /// if register `x` is greater or equal to the register `y`.
    /// Both qubit registers should be in a little-endian format.
    operation ApplyIfGreaterOrEqualLE<'T>(
        action : 'T => Unit is Adj + Ctl,
        x : Qubit[],
        y : Qubit[],
        target : 'T
    ) : Unit is Adj + Ctl {

        ApplyIfLessOrEqualLE(action, y, x, target);
    }

    /// # Summary
    /// Computes `if x > y { action(target) }`, that is, applies `action` to `target`
    /// if register `x` is greater than the register `y`.
    /// Both qubit registers should be in a little-endian format.
    operation ApplyIfGreaterLE<'T>(
        action : 'T => Unit is Adj + Ctl,
        x : Qubit[],
        y : Qubit[],
        target : 'T
    ) : Unit is Adj + Ctl {

        Fact(Length(x) > 0, "Bitwidth must be at least 1");
        within {
            ApplyToEachA(X, x);
        } apply {
            // control is inverted
            ApplyActionIfSumOverflows(action, x, y, true, target);
        }
    }

    export AddLE, ApplyIfEqualLE, ApplyIfEqualL, ApplyIfGreaterLE, ApplyIfGreaterL, ApplyIfGreaterOrEqualLE, ApplyIfGreaterOrEqualL, ApplyIfLessLE, ApplyIfLessL, ApplyIfLessOrEqualLE, ApplyIfLessOrEqualL, IncByI, IncByIUsingIncByLE, IncByL, IncByLUsingIncByLE, IncByLE, IncByLEUsingAddLE, LookAheadDKRSAddLE, MAJ, ReflectAboutInteger, RippleCarryCGAddLE, RippleCarryCGIncByLE, RippleCarryTTKIncByLE, FourierTDIncByLE;
}
