// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Arithmetic {
    open Microsoft.Quantum.Canon;
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Measurement;
    open Microsoft.Quantum.Math;

    /// # Summary
    /// Applies a bitwise-XOR operation between a classical integer and an
    /// integer represented by a register of qubits.
    ///
    /// # Description
    /// Applies `X` operations to qubits in a little-endian register based on
    /// 1 bits in an integer.
    ///
    /// Let us denote `value` by a and let y be an unsigned integer encoded in `target`,
    /// then `ApplyXorInPlace` performs an operation given by the following map:
    /// |y⟩ ↦ |y ⊕ a⟩, where ⊕ is the bitwise exclusive OR operator.
    operation ApplyXorInPlace(value : Int, target : Qubit[]) : Unit is Adj + Ctl {
        body (...) {
            Fact(value >= 0, "`value` must be non-negative.");
            mutable runningValue = value;
            for q in target {
                if (runningValue &&& 1) != 0 {
                    X(q);
                }
                set runningValue >>>= 1;
            }
            Fact(runningValue == 0, "value is too large");
        }
        adjoint self;
    }

    /// # Summary
    /// Applies a bitwise-XOR operation between a classical integer and an
    /// integer represented by a register of qubits.
    ///
    /// # Description
    /// Applies `X` operations to qubits in a little-endian register based on
    /// 1 bits in an integer.
    ///
    /// Let us denote `value` by a and let y be an unsigned integer encoded in `target`,
    /// then `ApplyXorInPlace` performs an operation given by the following map:
    /// |y⟩ ↦ |y ⊕ a⟩, where ⊕ is the bitwise exclusive OR operator.
    operation ApplyXorInPlaceL(value : BigInt, target : Qubit[]) : Unit is Adj + Ctl {
        body (...) {
            Fact(value >= 0L, "`value` must be non-negative.");
            mutable runningValue = value;
            for q in target {
                if (runningValue &&& 1L) != 0L {
                    X(q);
                }
                set runningValue >>>= 1;
            }
            Fact(runningValue == 0L, "`value` is too large.");
        }
        adjoint self;
    }

    /// # Summary
    /// Measures the content of a quantum register and converts
    /// it to an integer. The measurement is performed with respect
    /// to the standard computational basis, i.e., the eigenbasis of `PauliZ`.
    ///
    /// # Input
    /// ## target
    /// A quantum register in the little-endian encoding.
    ///
    /// # Output
    /// An unsigned integer that contains the measured value of `target`.
    ///
    /// # Remarks
    /// This operation resets its input register to the |00...0> state,
    /// suitable for releasing back to a target machine.
    @Config(Full)
    operation MeasureInteger(target : Qubit[]) : Int {
        let nBits = Length(target);
        Fact(nBits < 64, $"`Length(target)` must be less than 64, but was {nBits}.");

        mutable number = 0;
        for i in 0..nBits-1 {
            if (MResetZ(target[i]) == One) {
                set number |||= 1 <<< i;
            }
        }

        number
    }

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
    operation MAJ (x : Qubit, y : Qubit, z : Qubit) : Unit is Adj + Ctl {
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
    ///
    /// # Input
    /// ## index
    /// The classical integer j indexing the basis state about which to reflect.
    /// ## reg
    /// Little-endian quantum register to reflect.
    ///
    /// # Remarks
    /// This operation is implemented in-place, without explicit allocation of
    /// additional auxiliary qubits.
    operation ReflectAboutInteger (index : Int, reg : Qubit[]) : Unit is Adj + Ctl {
        within {
            // We want to reduce to the problem of reflecting about the all-ones
            // state. To do that, we apply our reflection within an application
            // of X instructions that flip all the zeros in our index.
            ApplyPauliFromInt(PauliX, false, index, reg);
        } apply {
            Controlled Z(Most(reg), Tail(reg));
        }
    }

    /// # Summary
    /// Automatically chooses between addition with
    /// carry and without, depending on the register size of `ys`,
    /// which holds the result after operation is complete.
    operation AddI (xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
        if Length(xs) == Length(ys) {
            RippleCarryAdderNoCarryTTK(xs, ys);
        }
        elif Length(ys) > Length(xs) {
            use qs = Qubit[Length(ys) - Length(xs) - 1];
            RippleCarryAdderTTK(xs + qs, Most(ys), Tail(ys));
        }
        else {
            fail "`xs` must not contain more qubits than `ys`.";
        }
    }

    /// # Summary
    /// Reversible, in-place ripple-carry addition of two integers without carry out.
    ///
    /// # Description
    /// Given two n-bit integers encoded in LittleEndian registers `xs` and `ys`,
    /// the operation computes the sum of the two integers modulo 2^n,
    /// where n is the length of the inputs arrays `xs` and `ys`,
    /// which must be positive. It does not compute the carry out bit.
    ///
    /// # Input
    /// ## xs
    /// LittleEndian qubit register encoding the first integer summand.
    /// ## ys
    /// LittleEndian qubit register encoding the second integer summand, is
    /// modified to hold the n least significant bits of the sum.
    ///
    /// # References
    /// - Yasuhiro Takahashi, Seiichiro Tani, Noboru Kunihiro: "Quantum
    ///   Addition Circuits and Unbounded Fan-Out", Quantum Information and
    ///   Computation, Vol. 10, 2010.
    ///   https://arxiv.org/abs/0910.2530
    ///
    /// # Remarks
    /// This operation has the same functionality as RippleCarryAdderTTK but does
    /// not return the carry bit.
    operation RippleCarryAdderNoCarryTTK(xs : Qubit[], ys : Qubit[])
    : Unit is Adj + Ctl {
        Fact(Length(xs) == Length(ys),
            "Input registers must have the same number of qubits." );
        Fact(Length(xs) > 0, "Array should not be empty.");

        if (Length(xs) > 1) {
            within {
                ApplyOuterTTKAdder(xs, ys);
            } apply {
                ApplyInnerTTKAdderWithoutCarry(xs, ys);
            }
        }
        CNOT (xs[0], ys[0]);
    }

    /// # Summary
    /// Reversible, in-place ripple-carry addition of two integers.
    ///
    /// # Description
    /// Given two n-bit integers encoded in LittleEndian registers `xs` and `ys`,
    /// and a qubit carry, the operation computes the sum of the two integers
    /// where the n least significant bits of the result are held in `ys` and
    /// the carry out bit is xored to the qubit `carry`.
    ///
    /// # Input
    /// ## xs
    /// LittleEndian qubit register encoding the first integer summand.
    /// ## ys
    /// LittleEndian qubit register encoding the second integer summand, is
    /// modified to hold the n least significant bits of the sum.
    /// ## carry
    /// Carry qubit, is xored with the carry out bit of the addition.
    ///
    /// # References
    /// - Yasuhiro Takahashi, Seiichiro Tani, Noboru Kunihiro: "Quantum
    ///   Addition Circuits and Unbounded Fan-Out", Quantum Information and
    ///   Computation, Vol. 10, 2010.
    ///   https://arxiv.org/abs/0910.2530
    ///
    /// # Remarks
    /// This operation has the same functionality as RippleCarryAdderD and,
    /// RippleCarryAdderCDKM but does not use any ancilla qubits.
    operation RippleCarryAdderTTK(xs : Qubit[], ys : Qubit[], carry : Qubit)
    : Unit is Adj + Ctl {
        Fact(Length(xs) == Length(ys),
            "Input registers must have the same number of qubits." );
        Fact(Length(xs) > 0, "Array should not be empty.");


        if (Length(xs) > 1) {
            CNOT(xs[Length(xs)-1], carry);
            within {
                ApplyOuterTTKAdder(xs, ys);
            } apply {
                ApplyInnerTTKAdder(xs, ys, carry);
            }
        }
        else {
            CCNOT(xs[0], ys[0], carry);
        }
        CNOT(xs[0], ys[0]);
    }

    /// # Summary
    /// Implements the outer operation for RippleCarryAdderTTK to conjugate
    /// the inner operation to construct the full adder. Input registers
    /// must be of the same size.
    ///
    /// # Input
    /// ## xs
    /// LittleEndian qubit register encoding the first integer summand
    /// input to RippleCarryAdderTTK.
    /// ## ys
    /// LittleEndian qubit register encoding the second integer summand
    /// input to RippleCarryAdderTTK.
    ///
    /// # References
    /// - Yasuhiro Takahashi, Seiichiro Tani, Noboru Kunihiro: "Quantum
    ///   Addition Circuits and Unbounded Fan-Out", Quantum Information and
    ///   Computation, Vol. 10, 2010.
    ///   https://arxiv.org/abs/0910.2530
    internal operation ApplyOuterTTKAdder(xs : Qubit[], ys : Qubit[])
    : Unit is Adj + Ctl {
        Fact(Length(xs) == Length(ys),
            "Input registers must have the same number of qubits." );
        for i in 1..Length(xs)-1 {
            CNOT(xs[i], ys[i]);
        }
        for i in Length(xs)-2..-1..1 {
            CNOT(xs[i], xs[i+1]);
        }
    }

    /// # Summary
    /// Implements the inner addition function for the operation
    /// RippleCarryAdderNoCarryTTK. This is the inner operation that is conjugated
    /// with the outer operation to construct the full adder.
    ///
    /// # Input
    /// ## xs
    /// LittleEndian qubit register encoding the first integer summand
    /// input to RippleCarryAdderNoCarryTTK.
    /// ## ys
    /// LittleEndian qubit register encoding the second integer summand
    /// input to RippleCarryAdderNoCarryTTK.
    ///
    /// # References
    /// - Yasuhiro Takahashi, Seiichiro Tani, Noboru Kunihiro: "Quantum
    ///   Addition Circuits and Unbounded Fan-Out", Quantum Information and
    ///   Computation, Vol. 10, 2010.
    ///   https://arxiv.org/abs/0910.2530
    ///
    /// # Remarks
    /// The specified controlled operation makes use of symmetry and mutual
    /// cancellation of operations to improve on the default implementation
    /// that adds a control to every operation.
    internal operation ApplyInnerTTKAdderWithoutCarry(xs : Qubit[], ys : Qubit[])
    : Unit is Adj + Ctl {
        body (...) {
            (Controlled ApplyInnerTTKAdderWithoutCarry) ([], (xs, ys));
        }
        controlled ( controls, ... ) {
            Fact(Length(xs) == Length(ys),
                "Input registers must have the same number of qubits." );

            for idx in 0..Length(xs) - 2 {
                CCNOT (xs[idx], ys[idx], xs[idx + 1]);
            }
            for idx in Length(xs)-1..-1..1 {
                Controlled CNOT(controls, (xs[idx], ys[idx]));
                CCNOT(xs[idx - 1], ys[idx - 1], xs[idx]);
            }
        }
    }

    /// # Summary
    /// Implements the inner addition function for the operation
    /// RippleCarryAdderTTK. This is the inner operation that is conjugated
    /// with the outer operation to construct the full adder.
    ///
    /// # Input
    /// ## xs
    /// LittleEndian qubit register encoding the first integer summand
    /// input to RippleCarryAdderTTK.
    /// ## ys
    /// LittleEndian qubit register encoding the second integer summand
    /// input to RippleCarryAdderTTK.
    /// ## carry
    /// Carry qubit, is xored with the most significant bit of the sum.
    ///
    /// # References
    /// - Yasuhiro Takahashi, Seiichiro Tani, Noboru Kunihiro: "Quantum
    ///   Addition Circuits and Unbounded Fan-Out", Quantum Information and
    ///   Computation, Vol. 10, 2010.
    ///   https://arxiv.org/abs/0910.2530
    ///
    /// # Remarks
    /// The specified controlled operation makes use of symmetry and mutual
    /// cancellation of operations to improve on the default implementation
    /// that adds a control to every operation.
    internal operation ApplyInnerTTKAdder(xs : Qubit[], ys : Qubit[], carry : Qubit)
    : Unit is Adj + Ctl {
        body (...) {
            (Controlled ApplyInnerTTKAdder)([], (xs, ys, carry));
        }
        controlled ( controls, ... ) {
            Fact(Length(xs) == Length(ys),
                "Input registers must have the same number of qubits." );
            Fact(Length(xs) > 0, "Array should not be empty.");


            let nQubits = Length(xs);
            for idx in 0..nQubits - 2 {
                CCNOT(xs[idx], ys[idx], xs[idx+1]);
            }
            (Controlled CCNOT)(controls, (xs[nQubits-1], ys[nQubits-1], carry));
            for idx in nQubits - 1..-1..1 {
                Controlled CNOT(controls, (xs[idx], ys[idx]));
                CCNOT(xs[idx-1], ys[idx-1], xs[idx]);
            }
        }
    }

    //
    //
    //      New arithmetic operations starts here.
    // Once it is done, previous implementation will be removed.
    //
    //

    //
    // Operation: Add      | General |    Ripple-carry    | Carry look-ahead |    Fourier
    // ____________________|_________|____________________|__________________|________________
    // y += 5              |  IncByL |  RippleCarryIncByL |                  |
    // y += x              | IncByLE | RippleCarryIncByLE | LookAheadIncByLE | FourierIncByLE
    // z = x + 5 (z was 0) |         |                    |                  |
    // z = x + y (z was 0) |   AddLE |   RippleCarryAddLE |   LookAheadAddLE |
    // z += x + 5          |         |                    |                  |
    // z += x + y          |         |                    |                  |
    //

    /// # Summary
    /// Increments a little-endian register ys by a BigInt number c
    ///
    /// # Description
    /// Computes ys += c modulo 2ⁿ, where ys is a little-endian register,
    /// Length(ys) = n > 0, c is a BigInt number, 0 ≤ c < 2ⁿ.
    /// NOTE: Use RippleCarryIncByL directly if the choice of implementation
    /// is important.
    operation IncByL (c : BigInt, ys : Qubit[]) : Unit is Adj + Ctl {
        RippleCarryIncByL(c, ys);
    }

    /// # Summary
    /// Increments a little-endian register ys by a little-endian register xs
    ///
    /// # Description
    /// Computes ys += xs modulo 2ⁿ, where xs and ys are little-endian registers,
    /// and Length(xs) ≤ Length(ys) = n.
    /// NOTE: Use RippleCarryIncByLE or LookAheadIncByLE directly if
    /// the choice of implementation is important.
    operation IncByLE (xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
        RippleCarryIncByLE(xs, ys);
    }

    /// # Summary
    /// Sets a zero-initialized little-endian register zs to the sum of
    /// little-endian registers xs and ys
    ///
    /// # Description
    /// Computes zs := xs + ys modulo 2ⁿ, where xs, ys, and zs are little-endian registers,
    /// Length(xs) = Length(ys) ≤ Length(zs) = n, assuming zs is 0-initialized.
    /// NOTE: Use RippleCarryAddLE or LookAheadAddLE directly if
    /// the choice of implementation is important.
    operation AddLE (xs : Qubit[], ys : Qubit[], zs : Qubit[]) : Unit is Adj {
        RippleCarryAddLE(xs, ys, zs);
    }

    /// # Summary
    /// Increments a little-endian register ys by a BigInt number c
    /// using the ripple-carry algorithm.
    ///
    /// # Description
    /// Computes ys += c modulo 2ⁿ, where ys is a little-endian register
    /// Length(ys) = n > 0, c is a BigInt number, 0 ≤ c < 2ⁿ.
    /// This operation uses the ripple-carry algorithm.
    ///
    /// # Reference
    ///     - [arXiv:1709.06648](https://arxiv.org/pdf/1709.06648.pdf)
    ///       "Halving the cost of quantum addition" by Craig Gidney.
    operation RippleCarryIncByL (c : BigInt, ys : Qubit[]) : Unit is Adj + Ctl {
        let ysLen = Length(ys);
        Fact(ysLen > 0, "Length of `ys` must be at least 1.");
        Fact(c >= 0L, "Constant `c` must be non-negative.");
        Fact(c < 2L^ysLen, "Constant `c` must be smaller than 2^Length(ys).");

        if c != 0L {
            // If c has j trailing zeroes than the j least significant
            // bits of y won't be affected by the addition and can
            // therefore be ignored by applying the addition only to
            // the other qubits and shifting c accordingly.
            let j = TrailingZeroCountL(c);
            use x = Qubit[ysLen - j];
            within {
                ApplyXorInPlaceL(c >>> j, x);
            } apply {
                RippleCarryIncByLE(x, ys[j...]);
            }
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
    ///     - [arXiv:1709.06648](https://arxiv.org/pdf/1709.06648.pdf)
    ///       "Halving the cost of quantum addition" by Craig Gidney.
    operation RippleCarryIncByLE (xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
        let xsLen = Length(xs);
        let ysLen = Length(ys);

        Fact(ysLen >= xsLen, "Register `ys` must be longer than register `xs`.");
        Fact(xsLen >= 1, "Registers `xs` and `ys` must contain at least one qubit.");

        if ysLen - xsLen >= 2 {
            use padding = Qubit[ysLen - xsLen - 1];
            RippleCarryIncByLE(xs + padding, ys);
        } elif xsLen == 1 {
            if ysLen == 1 {
                CNOT(xs[0], ys[0]);
            } elif ysLen == 2 {
                HalfAdderForInc(xs[0], ys[0], ys[1]);
            }
        } else {
            let (x0, xrest) = (Head(xs), Rest(xs));
            let (y0, yrest) = (Head(ys), Rest(ys));

            use carryOut = Qubit();
            within {
                ApplyAndAssuming0Target(x0, y0, carryOut);
            } apply {
                IncWithCarryIn(carryOut, xrest, yrest);
            }
            CNOT(x0, y0);
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
    ///     - [arXiv:1709.06648](https://arxiv.org/pdf/1709.06648.pdf)
    ///       "Halving the cost of quantum addition" by Craig Gidney.
    operation RippleCarryAddLE (xs : Qubit[], ys : Qubit[], zs : Qubit[]) : Unit is Adj {
        let xsLen = Length(xs);
        let zsLen = Length(zs);
        Fact(Length(ys) == xsLen, "Registers `xs` and `ys` must be of same length.");
        Fact(zsLen >= xsLen, "Register `zs` must be no shorter than register `xs`.");

        // Since zs is zero-initialized, its bits at indexes higher than
        // xsLen remain unsued as there will be no carry into them.
        let top = MinI(zsLen-2, xsLen-1);
        for k in 0 .. top {
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
    ///     - [arXiv:quant-ph/0406142](https://arxiv.org/abs/quant-ph/0406142)
    ///      "A logarithmic-depth quantum carry-lookahead adder" by
    ///      Thomas G. Draper, Samuel A. Kutin, Eric M. Rains, Krysta M. Svore
    operation LookAheadAddLE(xs : Qubit[], ys : Qubit[], zs : Qubit[]) : Unit is Adj {
        let xsLen = Length(xs);
        let zsLen = Length(zs);
        Fact(Length(ys) == xsLen, "Registers `xs` and `ys` must be of same length.");
        Fact(zsLen >= xsLen, "Register `zs` must be no shorter than register `xs`.");

        if zsLen > xsLen { // with carry-out
            // compute initial generate values
            for k in 0..xsLen - 1 {
                ApplyAndAssuming0Target(xs[k], ys[k], zs[k + 1]);
            }

            within {
                // compute initial propagate values
                ApplyToEachA(CNOT, Zipped(xs, ys));
            } apply {
                if xsLen > 1 {
                    ComputeCarries(Rest(ys), zs[1..xsLen]);
                }

                // compute sum into carries
                for k in 0..xsLen - 1 {
                    CNOT(ys[k], zs[k]);
                }
            }
        } else { // xsLen == zsLen, so without carry-out
            LookAheadAddLE(Most(xs), Most(ys), zs);
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
    ///     - [arXiv:quant-ph/0008033](https://arxiv.org/abs/quant-ph/0008033)
    ///      "Addition on a Quantum Computer" by Thomas G. Draper
    operation FourierIncByLE (xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
        within {
            ApplyQFT(ys);
        } apply {
            for (i, q) in Enumerated(xs) {
                Controlled PhaseGradient([q], ys[i...]);
            }
        }
    }

}
