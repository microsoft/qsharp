/// # Sample
/// Resource Estimation for Integer Factoring
///
/// # Description
/// In this sample we concentrate on costing quantum part in the algorithm for
/// factoring RSA integers based on Ekerå and Håstad
/// [ia.cr/2017/077](https://eprint.iacr.org/2017/077) based on the
/// implementation described in
/// [arXiv:1905.09749](https://arxiv.org/abs/1905.09749). This makes it ideal
/// for use with the Azure Quantum Resource Estimator.
namespace Microsoft.Quantum.Applications.Cryptography {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.ResourceEstimation;
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Diagnostics;

    // !!! IMPORTANT !!!
    // When computing resource estimtes from the VS Code plugin directly on this
    // file, make sure that you set the error budget to 0.333.

    @EntryPoint()
    operation EstimateEkeraHastad() : Unit {
        // Try different instances of the algorithm by commenting in and out
        // the following lines.  You can find more RSA numbers at
        // https://en.wikipedia.org/wiki/RSA_numbers

        // RSA-100 (330 bits)
        EkeraHastad(330, 1522605027922533360535618378132637429718068114961380688657908494580122963258952897654000350692006139L, 7L);

        // RSA-1024 (1024 bits)
        // EkeraHastad(1024, 135066410865995223349603216278805969938881475605667027524485143851526510604859533833940287150571909441798207282164471551373680419703964191743046496589274256239341020864383202110372958725762358509643110564073501508187510676594629205563685529475213500852879416377328533906109750544334999811150056977236890927563L, 7L);

        // RSA-2048 (2048 bits)
        // EkeraHastad(2048, 25195908475657893494027183240048398571429282126204032027777137836043662020707595556264018525880784406918290641249515082189298559149176184502808489120072844992687392807287776735971418347270261896375014971824691165077613379859095700097330459748808428401797429100642458691817195118746121515172654632282216869987549182422433637259085141865462043576798423387184774447920739934236584823824281198163815010674810451660377306056201619676256133844143603833904414952634432190114657544454178424020924616515723350778707749817125772467962926386356373289912154831438167899885040445364023527381951378636564391212010397122822120720357L, 7L);
    }

    /// # Summary
    /// Main algorithm based on quantum phase estimation
    ///
    /// # Reference
    /// [ia.cr/2017/077, Section 4.3](https://eprint.iacr.org/2017/077)
    operation EkeraHastad(numBits : Int, N : BigInt, g : BigInt) : Unit {
        let x = ExpModL(g, ((N - 1L) / 2L), N);
        let xinv = InverseModL(x, N);

        let m = numBits / 2;
        use c1 = Qubit[2 * m];
        use c2 = Qubit[m];
        use target = Qubit[numBits];

        let ne = 3 * m;
        let cpad = Ceiling(2.0 * Lg(IntAsDouble(numBits)) + Lg(IntAsDouble(ne)) + 10.0);

        // The algorithm uses the coset representation to replace modular
        // addition inside MultiplyExpMod with regular addition.
        use padding = Qubit[cpad];

        ApplyToEach(H, c1 + c2);
        InitCoset(N, target, padding);
        MultiplyExpMod(g, N, c1, target + padding);
        MultiplyExpMod(xinv, N, c2, target + padding);

        Adjoint ApplyQFT(c1);
        Adjoint ApplyQFT(c2);
    }

    // ------------------------------ //
    // Modular arithmetic (constants) //
    // ------------------------------ //

    /// Window size for exponentiation (c_exp)
    internal function ExponentWindowLength_() : Int { 5 }

    /// Window size for multiplication (c_mul)
    internal function MultiplicationWindowLength_() : Int { 5 }

    // ------------------------------- //
    // Modular arithmetic (operations) //
    // ------------------------------- //

    /// # Summary
    /// Encodes register in coset representation
    ///
    /// # Reference
    /// [arXiv:quant-ph/0601097, Section 4.1](https://arxiv.org/abs/quant-ph/0601097)
    internal operation InitCoset(mod : BigInt, xs : Qubit[], padding : Qubit[]) : Unit {
        use helper = Qubit();
        let cpad = Length(padding);
        let n = Length(xs);

        let combined = xs + padding;

        for j in 0..cpad - 1 {
            Controlled IncByLUsingIncByLE([helper], (RippleCarryCGIncByLE, mod, combined[j..j + n - 1]));

            ApplyIfLessOrEqualL(X, mod, combined[j..j + n - 1], helper);
        }
    }

    /// # Summary
    /// Computes zs *= (base ^ xs) % mod (for a large register xs)
    ///
    /// # Reference
    /// [arXiv:1905.07682, Fig. 7](https://arxiv.org/abs/1905.07682)
    internal operation MultiplyExpMod(
        base : BigInt,
        mod : BigInt,
        xs : Qubit[],
        zs : Qubit[]
    ) : Unit {
        let expWindows = Chunks(ExponentWindowLength_(), xs);

        within {
            RepeatEstimates(Length(expWindows));
        } apply {
            let i = 0; // in simulation this i must be iterated over IndexRange(expWindows)

            let adjustedBase = ExpModL(base, 1L <<< (i * ExponentWindowLength_()), mod);
            MultiplyExpModWindowed(adjustedBase, mod, expWindows[i], zs);
        }
    }

    /// # Summary
    /// Computes zs *= (base ^ xs) % mod (for a small register xs)
    ///
    /// # Reference
    /// [arXiv:1905.07682, Fig. 6](https://arxiv.org/abs/1905.07682)
    internal operation MultiplyExpModWindowed(
        base : BigInt,
        mod : BigInt,
        xs : Qubit[],
        zs : Qubit[]
    ) : Unit {
        let n = Length(zs);

        use qs = Qubit[n];
        AddExpModWindowed(base, mod, 1, xs, zs, qs);
        AddExpModWindowed(InverseModL(base, mod), mod, -1, xs, qs, zs);
        for i in IndexRange(zs) {
            SWAP(zs[i], qs[i]);
        }
    }

    /// # Summary
    /// Computes zs += ys * (base ^ xs) % mod (for small registers xs and ys)
    ///
    /// # Reference
    /// [arXiv:1905.07682, Fig. 5](https://arxiv.org/abs/1905.07682)
    ///
    /// # Remark
    /// Unlike in the reference, this implementation uses regular addition
    /// instead of modular addition because the target register is encoded
    /// using the coset representation.
    internal operation AddExpModWindowed(
        base : BigInt,
        mod : BigInt,
        sign : Int,
        xs : Qubit[],
        ys : Qubit[],
        zs : Qubit[]
    ) : Unit {
        // split factor into parts
        let factorWindows = Chunks(MultiplicationWindowLength_(), ys);

        within {
            RepeatEstimates(Length(factorWindows));
        } apply {
            let i = 0; // in simulation this i must be iterated over IndexRange(factorWindows)

            // compute data for table lookup
            let factorValue = ExpModL(2L, IntAsBigInt(i * MultiplicationWindowLength_()), mod);
            let data = LookupData(factorValue, Length(xs), Length(factorWindows[i]), base, mod, sign, Length(zs));

            use output = Qubit[Length(data[0])];

            within {
                Select(data, xs + factorWindows[i], output);
            } apply {
                RippleCarryCGIncByLE(output, zs);
            }
        }
    }

    internal function LookupData(factor : BigInt, expLength : Int, mulLength : Int, base : BigInt, mod : BigInt, sign : Int, numBits : Int) : Bool[][] {
        mutable data = [[false, size = numBits], size = 2^(expLength + mulLength)];
        for b in 0..2^mulLength - 1 {
            for a in 0..2^expLength - 1 {
                let idx = b * 2^expLength + a;
                let value = ModulusL(factor * IntAsBigInt(b) * IntAsBigInt(sign) * (base^a), mod);
                set data w/= idx <- BigIntAsBoolArray(value, numBits);
            }
        }

        data
    }

    // -- library code from `Unstable` below --

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

/// # Summary
    /// Performs table lookup using a SELECT network
    ///
    /// # Description
    /// Assuming a zero-initialized `target` register, this operation will
    /// initialize it with the bitstrings in `data` at indices according to the
    /// computational values of the `address` register.
    ///
    /// # Input
    /// ## data
    /// The classical table lookup data which is prepared in `target` with
    /// respect to the state in `address`. The length of data must be less than
    /// 2ⁿ, where 𝑛 is the length of `address`. Each entry in data must have
    /// the same length that must be equal to the length of `target`.
    /// ## address
    /// Address register
    /// ## target
    /// Zero-initialized target register
    ///
    /// # Remarks
    /// The implementation of the SELECT network is based on unary encoding as
    /// presented in [1].  The recursive implementation of that algorithm is
    /// presented in [3].  The adjoint variant is optimized using a
    /// measurement-based unlookup operation [3]. The controlled adjoint variant
    /// is not optimized using this technique.
    ///
    /// # References
    /// 1. [arXiv:1805.03662](https://arxiv.org/abs/1805.03662)
    ///    "Encoding Electronic Spectra in Quantum Circuits with Linear T
    ///    Complexity"
    /// 2. [arXiv:1905.07682](https://arxiv.org/abs/1905.07682)
    ///    "Windowed arithmetic"
    /// 3. [arXiv:2211.01133](https://arxiv.org/abs/2211.01133)
    ///    "Space-time optimized table lookup"
    operation Select(
        data : Bool[][],
        address : Qubit[],
        target : Qubit[]
    ) : Unit is Adj + Ctl {
        body (...) {
            let (N, n) = DimensionsForSelect(data, address);

            if N == 1 {
                // base case
                WriteMemoryContents(Head(data), target);
            } else {
                let (most, tail) = MostAndTail(address[...n - 1]);
                let parts = Partitioned([2^(n - 1)], data);

                within {
                    X(tail);
                } apply {
                    SinglyControlledSelect(tail, parts[0], most, target);
                }

                SinglyControlledSelect(tail, parts[1], most, target);
            }
        }
        adjoint (...) {
            Unlookup(Select, data, address, target);
        }

        controlled (ctls, ...) {
            let numCtls = Length(ctls);

            if numCtls == 0 {
                Select(data, address, target);
            } elif numCtls == 1 {
                SinglyControlledSelect(ctls[0], data, address, target);
            } else {
                use andChainTarget = Qubit();
                let andChain = MakeAndChain(ctls, andChainTarget);
                use helper = Qubit[andChain::NGarbageQubits];

                within {
                    andChain::Apply(helper);
                } apply {
                    SinglyControlledSelect(andChainTarget, data, address, target);
                }
            }
        }

        controlled adjoint (ctls, ...) {
            Controlled Select(ctls, (data, address, target));
        }
    }

    operation SinglyControlledSelect(
        ctl : Qubit,
        data : Bool[][],
        address : Qubit[],
        target : Qubit[]
    ) : Unit {
        let (N, n) = DimensionsForSelect(data, address);

        if BeginEstimateCaching("Unstable.TableLookup.SinglyControlledSelect", N) {
            if N == 1 {
                // base case
                Controlled WriteMemoryContents([ctl], (Head(data), target));
            } else {
                use helper = Qubit();

                let (most, tail) = MostAndTail(address[...n - 1]);
                let parts = Partitioned([2^(n - 1)], data);

                within {
                    X(tail);
                } apply {
                    ApplyAndAssuming0Target(ctl, tail, helper);
                }

                SinglyControlledSelect(helper, parts[0], most, target);

                CNOT(ctl, helper);

                SinglyControlledSelect(helper, parts[1], most, target);

                Adjoint ApplyAndAssuming0Target(ctl, tail, helper);
            }

            EndEstimateCaching();
        }
    }

    function DimensionsForSelect(
        data : Bool[][],
        address : Qubit[]
    ) : (Int, Int) {
        let N = Length(data);
        Fact(N > 0, "data cannot be empty");

        let n = Ceiling(Lg(IntAsDouble(N)));
        Fact(
            Length(address) >= n,
            $"address register is too small, requires at least {n} qubits"
        );

        return (N, n);
    }

    operation WriteMemoryContents(
        value : Bool[],
        target : Qubit[]
    ) : Unit is Adj + Ctl {
        Fact(
            Length(value) == Length(target),
            "number of data bits must equal number of target qubits"
        );

        ApplyPauliFromBitString(PauliX, true, value, target);
    }

    /// # References
    /// - [arXiv:1905.07682](https://arxiv.org/abs/1905.07682)
    ///   "Windowed arithmetic"
    operation Unlookup(
        lookup : (Bool[][], Qubit[], Qubit[]) => Unit,
        data : Bool[][],
        select : Qubit[],
        target : Qubit[]
    ) : Unit {
        let numBits = Length(target);
        let numAddressBits = Length(select);

        let l = MinI(Floor(Lg(IntAsDouble(numBits))), numAddressBits - 1);
        Fact(
            l < numAddressBits,
            $"l = {l} must be smaller than {numAddressBits}"
        );

        let res = Mapped(r -> r == One, ForEach(MResetX, target));

        let dataFixup = Chunks(2^l, Padded(-2^numAddressBits, false, Mapped(MustBeFixed(res, _), data)));

        let numAddressBitsFixup = numAddressBits - l;

        let selectParts = Partitioned([l], select);
        let targetFixup = target[...2^l - 1];

        within {
            EncodeUnary(selectParts[0], targetFixup);
            ApplyToEachA(H, targetFixup);
        } apply {
            lookup(dataFixup, selectParts[1], targetFixup);
        }
    }

    // Checks whether specific bit string `data` must be fixed for a given
    // measurement result `result`.
    //
    // Returns true if the number of indices for which both result and data are
    // `true` is odd.
    function MustBeFixed(result : Bool[], data : Bool[]) : Bool {
        mutable state = false;
        for i in IndexRange(result) {
            set state = state != (result[i] and data[i]);
        }
        state
    }

    // Computes unary encoding of value in `input` into `target`
    //
    // Assumptions:
    //    - `target` is zero-initialized
    //    - length of `input` is n
    //    - length of `target` is 2^n
    operation EncodeUnary(
        input : Qubit[],
        target : Qubit[]
    ) : Unit is Adj {
        Fact(
            Length(target) == 2^Length(input),
            $"target register should be of length {2^Length(input)}, but is {Length(target)}"
        );

        X(Head(target));

        for i in IndexRange(input) {
            if i == 0 {
                CNOT(input[i], target[1]);
                CNOT(target[1], target[0]);
            } else {
                // targets are the first and second 2^i qubits of the target register
                let split = Partitioned([2^i, 2^i], target);
                for j in IndexRange(split[0]) {
                    ApplyAndAssuming0Target(input[i], split[0][j], split[1][j]);
                    CNOT(split[1][j], split[0][j]);
                }
            }
        }

    }

    newtype AndChain = (
        NGarbageQubits : Int,
        Apply : Qubit[] => Unit is Adj
    );

    function MakeAndChain(ctls : Qubit[], target : Qubit) : AndChain {
        AndChain(
            MaxI(Length(ctls) - 2, 0),
            helper => AndChainOperation(ctls, helper, target)
        )
    }

    operation AndChainOperation(ctls : Qubit[], helper : Qubit[], target : Qubit) : Unit is Adj {
        let n = Length(ctls);

        Fact(Length(helper) == MaxI(n - 2, 0), "Invalid number of helper qubits");

        if n == 0 {
            X(target);
        } elif n == 1 {
            CNOT(ctls[0], target);
        } else {
            let ctls1 = ctls[0..0] + helper;
            let ctls2 = ctls[1...];
            let tgts = helper + [target];

            for idx in IndexRange(tgts) {
                ApplyAndAssuming0Target(ctls1[idx], ctls2[idx], tgts[idx]);
            }
        }
    }

/// # Summary
    /// Implements the outer operation for RippleCarryTTKIncByLE to conjugate
    /// the inner operation to construct the full adder. Only Length(xs)
    /// qubits are processed.
    ///
    /// # Input
    /// ## xs
    /// Qubit register in a little-endian format containing the first summand
    /// input to RippleCarryTTKIncByLE.
    /// ## ys
    /// Qubit register in a little-endian format containing the second summand
    /// input to RippleCarryTTKIncByLE.
    ///
    /// # References
    /// - Yasuhiro Takahashi, Seiichiro Tani, Noboru Kunihiro: "Quantum
    ///   Addition Circuits and Unbounded Fan-Out", Quantum Information and
    ///   Computation, Vol. 10, 2010.
    ///   https://arxiv.org/abs/0910.2530
    operation ApplyOuterTTKAdder(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
        Fact(Length(xs) <= Length(ys), "Input register ys must be at lease as long as xs.");
        for i in 1..Length(xs) - 1 {
            CNOT(xs[i], ys[i]);
        }
        for i in Length(xs) - 2..-1..1 {
            CNOT(xs[i], xs[i + 1]);
        }
    }

    /// # Summary
    /// Implements the inner addition function for the operation
    /// RippleCarryTTKIncByLE. This is the inner operation that is conjugated
    /// with the outer operation to construct the full adder.
    ///
    /// # Input
    /// ## xs
    /// Qubit register in a little-endian format containing the first summand
    /// input to RippleCarryTTKIncByLE.
    /// ## ys
    /// Qubit register in a little-endian format containing the second summand
    /// input to RippleCarryTTKIncByLE.
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
    operation ApplyInnerTTKAdderNoCarry(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
        body (...) {
            (Controlled ApplyInnerTTKAdderNoCarry)([], (xs, ys));
        }
        controlled (controls, ...) {
            Fact(Length(xs) == Length(ys), "Input registers must have the same number of qubits.");

            for idx in 0..Length(xs) - 2 {
                CCNOT(xs[idx], ys[idx], xs[idx + 1]);
            }
            for idx in Length(xs) - 1..-1..1 {
                Controlled CNOT(controls, (xs[idx], ys[idx]));
                CCNOT(xs[idx - 1], ys[idx - 1], xs[idx]);
            }
        }
    }

    /// # Summary
    /// Implements the inner addition function for the operation
    /// RippleCarryTTKIncByLE. This is the inner operation that is conjugated
    /// with the outer operation to construct the full adder.
    ///
    /// # Input
    /// ## xs
    /// Qubit register in a little-endian format containing the first summand
    /// input to RippleCarryTTKIncByLE.
    /// ## ys
    /// Qubit register in a little-endian format containing the second summand
    /// input to RippleCarryTTKIncByLE.
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
    operation ApplyInnerTTKAdderWithCarry(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
        body (...) {
            (Controlled ApplyInnerTTKAdderWithCarry)([], (xs, ys));
        }
        controlled (controls, ...) {
            Fact(Length(xs) + 1 == Length(ys), "ys must be one qubit longer then xs.");
            Fact(Length(xs) > 0, "Array should not be empty.");


            let nQubits = Length(xs);
            for idx in 0..nQubits - 2 {
                CCNOT(xs[idx], ys[idx], xs[idx + 1]);
            }
            (Controlled CCNOT)(controls, (xs[nQubits - 1], ys[nQubits - 1], ys[nQubits]));
            for idx in nQubits - 1..-1..1 {
                Controlled CNOT(controls, (xs[idx], ys[idx]));
                CCNOT(xs[idx - 1], ys[idx - 1], xs[idx]);
            }
        }
    }

    /// # Summary
    /// Implements Half-adder. Adds qubit x to qubit y and sets carryOut appropriately
    operation HalfAdderForInc(x : Qubit, y : Qubit, carryOut : Qubit) : Unit is Adj + Ctl {
        body (...) {
            CCNOT(x, y, carryOut);
            CNOT(x, y);
        }
        adjoint auto;

        controlled (ctls, ...) {
            Fact(Length(ctls) == 1, "HalfAdderForInc should be controlled by exactly one control qubit.");

            let ctl = ctls[0];
            use helper = Qubit();

            within {
                ApplyAndAssuming0Target(x, y, helper);
            } apply {
                ApplyAndAssuming0Target(ctl, helper, carryOut);
            }
            CCNOT(ctl, x, y);
        }
        controlled adjoint auto;
    }

    /// # Summary
    /// Implements Full-adder. Adds qubit carryIn and x to qubit y and sets carryOut appropriately.
    operation FullAdderForInc(carryIn : Qubit, x : Qubit, y : Qubit, carryOut : Qubit) : Unit is Adj + Ctl {
        body (...) {
            // TODO: cannot use `Carry` operation here
            CNOT(carryIn, x);
            CNOT(carryIn, y);
            CCNOT(x, y, carryOut);
            CNOT(carryIn, carryOut);
            CNOT(carryIn, x);
            CNOT(x, y);
        }
        adjoint auto;

        controlled (ctls, ...) {
            Fact(Length(ctls) == 1, "FullAdderForInc should be controlled by exactly one control qubit.");

            let ctl = ctls[0];
            use helper = Qubit();

            CarryForInc(carryIn, x, y, helper);
            CCNOT(ctl, helper, carryOut);
            Controlled UncarryForInc(ctls, (carryIn, x, y, helper));
        }
        controlled adjoint auto;
    }

    // Computes carryOut := carryIn + x + y
    operation FullAdder(carryIn : Qubit, x : Qubit, y : Qubit, carryOut : Qubit) : Unit is Adj {
        CNOT(x, y);
        CNOT(x, carryIn);
        ApplyAndAssuming0Target(y, carryIn, carryOut);
        CNOT(x, y);
        CNOT(x, carryOut);
        CNOT(y, carryIn);
    }

    /// # Summary
    /// Computes carry bit for a full adder.
    operation CarryForInc(carryIn : Qubit, x : Qubit, y : Qubit, carryOut : Qubit) : Unit is Adj + Ctl {
        body (...) {
            CNOT(carryIn, x);
            CNOT(carryIn, y);
            ApplyAndAssuming0Target(x, y, carryOut);
            CNOT(carryIn, carryOut);
        }
        adjoint auto;
        controlled (ctls, ...) {
            // This CarryForInc is intended to be used only in an in-place
            // ripple-carry implementation. Only such particular use case allows
            // for this simple implementation where controlled version
            // is the same as uncontrolled body.
            CarryForInc(carryIn, x, y, carryOut);
        }
        controlled adjoint auto;
    }

    /// # Summary
    /// Uncomputes carry bit for a full adder.
    operation UncarryForInc(carryIn : Qubit, x : Qubit, y : Qubit, carryOut : Qubit) : Unit is Adj + Ctl {
        body (...) {
            CNOT(carryIn, carryOut);
            Adjoint ApplyAndAssuming0Target(x, y, carryOut);
            CNOT(carryIn, x);
            CNOT(x, y);
        }
        adjoint auto;
        controlled (ctls, ...) {
            Fact(Length(ctls) == 1, "UncarryForInc should be controlled by exactly one control qubit.");

            let ctl = ctls[0];

            CNOT(carryIn, carryOut);
            Adjoint ApplyAndAssuming0Target(x, y, carryOut);
            CCNOT(ctl, x, y); // Controlled X(ctls + [x], y);
            CNOT(carryIn, x);
            CNOT(carryIn, y);
        }
        controlled adjoint auto;
    }

    /// # Summary
    /// Applies AND gate between `control1` and `control2` and stores the result
    /// in `target` assuming `target` is in |0> state.
    ///
    /// # Description
    /// Inverts `target` if and only if both controls are 1, but assumes that
    /// `target` is in state 0. The operation has T-count 4, T-depth 2 and
    /// requires no helper qubit, and may therefore be preferable to a CCNOT
    /// operation, if `target` is known to be 0.
    /// The adjoint of this operation is measurement based and requires no T
    /// gates (but requires target to support branching on measurements).
    /// Although the Toffoli gate (CCNOT) will perform faster in simulations,
    /// this version has lower T gate requirements.
    /// # References
    /// - Cody Jones: "Novel constructions for the fault-tolerant Toffoli gate",
    ///   Phys. Rev. A 87, 022328, 2013
    ///   [arXiv:1212.5069](https://arxiv.org/abs/1212.5069)
    ///   doi:10.1103/PhysRevA.87.022328
    @Config(Adaptive)
    operation ApplyAndAssuming0Target(control1 : Qubit, control2 : Qubit, target : Qubit) : Unit is Adj {
        // NOTE: Eventually this operation will be public and intrinsic.
        body (...) {
            CCNOT(control1, control2, target);
        }
        adjoint (...) {
            H(target);
            if M(target) == One {
                Reset(target);
                CZ(control1, control2);
            }
        }
    }

    operation ApplyOrAssuming0Target(control1 : Qubit, control2 : Qubit, target : Qubit) : Unit is Adj {
        within {
            X(control1);
            X(control2);
        } apply {
            ApplyAndAssuming0Target(control1, control2, target);
            X(target);
        }
    }

    /// # Summary
    /// Applies AND gate between `control1` and `control2` and stores the result
    /// in `target` assuming `target` is in |0> state.
    ///
    /// # Description
    /// Inverts `target` if and only if both controls are 1, but assumes that
    /// `target` is in state 0. The operation has T-count 4, T-depth 2 and
    /// requires no helper qubit, and may therefore be preferable to a CCNOT
    /// operation, if `target` is known to be 0.
    /// This version is suitable for Base profile.
    /// Although the Toffoli gate (CCNOT) will perform faster in simulations,
    /// this version has lower T gate requirements.
    /// # References
    /// - Cody Jones: "Novel constructions for the fault-tolerant Toffoli gate",
    ///   Phys. Rev. A 87, 022328, 2013
    ///   [arXiv:1212.5069](https://arxiv.org/abs/1212.5069)
    ///   doi:10.1103/PhysRevA.87.022328
    @Config(not Adaptive)
    operation ApplyAndAssuming0Target(control1 : Qubit, control2 : Qubit, target : Qubit) : Unit is Adj {
        H(target);
        T(target);
        CNOT(control1, target);
        CNOT(control2, target);
        within {
            CNOT(target, control1);
            CNOT(target, control2);
        } apply {
            Adjoint T(control1);
            Adjoint T(control2);
            T(target);
        }
        H(target);
        S(target);
    }

    /// # Summary
    /// Computes carries for the look-ahead adder
    operation ComputeCarries(ps : Qubit[], gs : Qubit[]) : Unit is Adj {
        let n = Length(gs);
        Fact(Length(ps) + 1 == n, "Register gs must be one qubit longer than register gs.");

        let T = Floor(Lg(IntAsDouble(n)));
        use qs = Qubit[n - HammingWeightI(n) - T];

        let registerPartition = MappedOverRange(t -> Floor(IntAsDouble(n) / IntAsDouble(2^t)) - 1, 1..T - 1);
        let pWorkspace = [ps] + Partitioned(registerPartition, qs);

        within {
            PRounds(pWorkspace);
        } apply {
            // U_G
            GRounds(pWorkspace, gs);

            // U_C
            CRounds(pWorkspace, gs);
        }
    }

    /// # Summary
    /// Computes all p[i, j] values in workspace for the look-ahead adder.
    ///
    /// The register array `pWorkspace` has T entries, where T = ⌊log₂ n⌋.
    ///
    /// The first entry `pWorkspace[0]` is initialized with `P_0` which is
    /// computed before `ComputeCarries` is called.  The other registers are
    /// 0-initialized and will be computed in successive rounds t = 1, ..., T - 1.
    ///
    /// In each round t we compute
    ///
    /// p[i, j] = p[2ᵗ × m, 2ᵗ × (m + 1)] = p[i, k] ∧ p[k, j]
    ///
    /// in `pWorkspace[t][m - 1]` and use that for k = 2ᵗ × m + 2ᵗ⁻¹, p[i, k] and p[k, j]
    /// have already been computed in round t - 1 in `pWorkspace[t - 1][2 * m - 1]` and
    /// `pWorkspace[t - 1][2 * m]`, respectively.
    operation PRounds(pWorkspace : Qubit[][]) : Unit is Adj {
        for ws in Windows(2, pWorkspace) {
            // note that we are using Rest, since pWorkspace[t - 1][0] is never
            // accessed in round t.
            let (current, next) = (Rest(ws[0]), ws[1]);

            for m in IndexRange(next) {
                ApplyAndAssuming0Target(current[2 * m], current[2 * m + 1], next[m]);
            }
        }
    }

    /// # Summary
    /// Computes g[i ∧ (i + 1), i + 1] into gs[i] for the look-ahead adder.
    ///
    /// The register gs has n entries initialized to gs[i] = g[i, i + 1].
    ///
    /// After successive rounds t = 1, ..., T, the register is updated to
    /// gs[i] = g[i ∧ (i + 1), i + 1], from which we can compute the carries
    /// in the C-rounds.
    operation GRounds(pWorkspace : Qubit[][], gs : Qubit[]) : Unit is Adj {
        let T = Length(pWorkspace);
        let n = Length(gs);

        for t in 1..T {
            let length = Floor(IntAsDouble(n) / IntAsDouble(2^t)) - 1;
            let ps = pWorkspace[t - 1][0..2...];

            for m in 0..length {
                CCNOT(gs[2^t * m + 2^(t - 1) - 1], ps[m], gs[2^t * m + 2^t - 1]);
            }
        }
    }

    /// # Summary
    /// Computes carries into gs for the look-ahead adder.
    operation CRounds(pWorkspace : Qubit[][], gs : Qubit[]) : Unit is Adj {
        let n = Length(gs);

        let start = Floor(Lg(IntAsDouble(2 * n) / 3.0));
        for t in start..-1..1 {
            let length = Floor(IntAsDouble(n - 2^(t - 1)) / IntAsDouble(2^t));
            let ps = pWorkspace[t - 1][1..2...];

            for m in 1..length {
                CCNOT(gs[2^t * m - 1], ps[m - 1], gs[2^t * m + 2^(t - 1) - 1]);
            }
        }
    }

    operation PhaseGradient(qs : Qubit[]) : Unit is Adj + Ctl {
        for i in IndexRange(qs) {
            R1Frac(1, i, qs[i]);
        }
    }

    //
    // operations for comparisons
    //

    /// # Summary
    /// Applies `action` to `target` if register `x` is greater or equal to BigInt `c`
    /// (if `invertControl` is false). If `invertControl` is true, the `action`
    /// is applied in the opposite situation.
    operation ApplyActionIfGreaterThanOrEqualConstant<'T>(
        invertControl : Bool,
        action : 'T => Unit is Adj + Ctl,
        c : BigInt,
        x : Qubit[],
        target : 'T
    ) : Unit is Adj + Ctl {

        let bitWidth = Length(x);
        if c == 0L {
            if not invertControl {
                action(target);
            }
        } elif c >= (2L^bitWidth) {
            if invertControl {
                action(target);
            }
        } else {
            // normalize constant
            let l = TrailingZeroCountL(c);

            let cNormalized = c >>> l;
            let xNormalized = x[l...];
            let bitWidthNormalized = Length(xNormalized);

            // If c == 2L^(bitwidth - 1), then bitWidthNormalized will be 1,
            // and qs will be empty.  In that case, we do not need to compute
            // any temporary values, and some optimizations are apply, which
            // are considered in the remainder.
            use qs = Qubit[bitWidthNormalized - 1];
            let cs1 = IsEmpty(qs) ? [] | [Head(xNormalized)] + Most(qs);

            Fact(Length(cs1) == Length(qs), "Arrays should be of the same length.");

            within {
                for i in 0..Length(cs1) - 1 {
                    let op = cNormalized &&& (1L <<< (i + 1)) != 0L ? ApplyAndAssuming0Target | ApplyOrAssuming0Target;
                    op(cs1[i], xNormalized[i + 1], qs[i]);
                }
            } apply {
                let control = IsEmpty(qs) ? Tail(x) | Tail(qs);
                within {
                    if invertControl {
                        X(control);
                    }
                } apply {
                    Controlled action([control], target);
                }
            }
        }
    }

/// # Summary
    /// Applies `action` to `target` if the sum of `x` and `y` registers
    /// overflows, i.e. there's a carry out (if `invertControl` is false).
    /// If `invertControl` is true, the `action` is applied when there's no carry out.
    operation ApplyActionIfSumOverflows<'T>(
        action : 'T => Unit is Adj + Ctl,
        x : Qubit[],
        y : Qubit[],
        invertControl : Bool,
        target : 'T
    ) : Unit is Adj + Ctl {

        let n = Length(x);
        Fact(n >= 1, "Registers must contain at least one qubit.");
        Fact(Length(y) == n, "Registers must be of the same length.");

        use carries = Qubit[n];

        within {
            CarryWith1CarryIn(x[0], y[0], carries[0]);
            for i in 1..n - 1 {
                CarryForInc(carries[i - 1], x[i], y[i], carries[i]);
            }
        } apply {
            within {
                if invertControl {
                    X(carries[n - 1]);
                }
            } apply {
                Controlled action([carries[n - 1]], target);
            }
        }
    }

/// # Summary
    /// Computes carry out assuming carry in is 1.
    /// Simplified version that is only applicable for scenarios
    /// where controlled version is the same as non-controlled.
    operation CarryWith1CarryIn(
        x : Qubit,
        y : Qubit,
        carryOut : Qubit
    ) : Unit is Adj + Ctl {

        body (...) {
            X(x);
            X(y);
            ApplyAndAssuming0Target(x, y, carryOut);
            X(carryOut);
        }

        adjoint auto;

        controlled (ctls, ...) {
            Fact(Length(ctls) <= 1, "Number of control lines must be at most 1");
            CarryWith1CarryIn(x, y, carryOut);
        }

        controlled adjoint auto;
    }

    /// # Summary
    /// This wrapper allows operations that support only one control
    /// qubit to be used in a multi-controlled scenarios. It provides
    /// controlled version that collects controls into one qubit
    /// by applying AND chain using auxiliary qubit array.
    operation ApplyAsSinglyControlled<'TIn>(
        op : ('TIn => Unit is Adj + Ctl),
        input : 'TIn
    ) : Unit is Adj + Ctl {

        body (...) {
            op(input);
        }

        controlled (ctls, ...) {
            let n = Length(ctls);
            if n == 0 {
                op(input);
            } elif n == 1 {
                Controlled op(ctls, input);
            } else {
                use aux = Qubit[n - 1];
                within {
                    LogDepthAndChain(ctls, aux);
                } apply {
                    Controlled op([Tail(aux)], input);
                }
            }
        }
    }

/// # Summary
    /// This helper function computes the AND of all control bits in `ctls` into
    /// the last qubit of `tgts`, using the other qubits in `tgts` as helper
    /// qubits for the AND of subsets of control bits.  The operation has a
    /// logarithmic depth of AND gates by aligning them using a balanced binary
    /// tree.
    operation LogDepthAndChain(ctls : Qubit[], tgts : Qubit[]) : Unit is Adj {
        let lc = Length(ctls);
        let lt = Length(tgts);

        Fact(lc == lt + 1, $"There must be exactly one more control qubit than target qubits (got {lc}, {lt})");

        if lt == 1 {
            ApplyAndAssuming0Target(ctls[0], ctls[1], tgts[0]);
        } elif lt == 2 {
            ApplyAndAssuming0Target(ctls[0], ctls[1], tgts[0]);
            ApplyAndAssuming0Target(ctls[2], tgts[0], tgts[1]);
        } else {
            let left = lc / 2;
            let right = lc - left;

            let ctlsLeft = ctls[...left - 1];
            let tgtsLeft = tgts[...left - 2];

            let ctlsRight = ctls[left..left + right - 1];
            let tgtsRight = tgts[left - 1..left + right - 3];

            LogDepthAndChain(ctlsLeft, tgtsLeft);
            LogDepthAndChain(ctlsRight, tgtsRight);
            ApplyAndAssuming0Target(Tail(tgtsLeft), Tail(tgtsRight), Tail(tgts));
        }
    }


}
