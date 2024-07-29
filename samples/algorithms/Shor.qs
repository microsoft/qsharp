/// # Sample
/// Shor's algorithm
///
/// # Description
/// Shor's algorithm is a quantum algorithm for finding the prime factors of an
/// integer.
///
/// This Q# program implements Shor's algorithm.
namespace Sample {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Random;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Arrays;

    @EntryPoint()
    operation Main() : (Int, Int) {
        let n = 143; // 11*13;
        // You can try these other examples for a lengthier computation.
        // let n = 16837; // = 113*149
        // let n = 22499; // = 149*151

        // Use Shor's algorithm to factor a semiprime integer.
        let (a, b) = FactorSemiprimeInteger(n);
        Message($"Found factorization {n} = {a} * {b}");
        return (a, b);
    }

    /// # Summary
    /// Uses Shor's algorithm to factor an input number.
    ///
    /// # Input
    /// ## number
    /// A semiprime integer to be factored.
    ///
    /// # Output
    /// Pair of numbers p > 1 and q > 1 such that p⋅q = `number`
    operation FactorSemiprimeInteger(number : Int) : (Int, Int) {
        // First check the most trivial case (the provided number is even).
        if number % 2 == 0 {
            Message("An even number has been given; 2 is a factor.");
            return (number / 2, 2);
        }
        // These mutables will keep track of whether we found the factors, and
        // if so, what they are. The default value for the factors is (1,1).
        mutable foundFactors = false;
        mutable factors = (1, 1);
        mutable attempt = 1;
        repeat {
            Message($"*** Factorizing {number}, attempt {attempt}.");
            // Try to guess a number co-prime to `number` by getting a random
            // integer in the interval [1, number-1]
            let generator = DrawRandomInt(1, number - 1);

            // Check if the random integer is indeed co-prime.
            // If true use Quantum algorithm for Period finding.
            if GreatestCommonDivisorI(generator, number) == 1 {
                Message($"Estimating period of {generator}.");

                // Call Quantum Period finding algorithm for
                // `generator` mod `number`.
                let period = EstimatePeriod(generator, number);

                // Set the flag and factors values if the continued
                // fractions classical algorithm succeeds.
                set (foundFactors, factors) = MaybeFactorsFromPeriod(number, generator, period);
            }
            // In this case, we guessed a divisor by accident.
            else {
                // Find divisor.
                let gcd = GreatestCommonDivisorI(number, generator);
                Message($"We have guessed a divisor {gcd} by accident. " + "No quantum computation was done.");

                // Set the flag `foundFactors` to true, indicating that we
                // succeeded in finding factors.
                set foundFactors = true;
                set factors = (gcd, number / gcd);
            }
            set attempt = attempt + 1;
            if (attempt > 100) {
                fail "Failed to find factors: too many attempts!";
            }
        } until foundFactors
        fixup {
            Message("The estimated period did not yield a valid factor. " + "Trying again.");
        }

        // Return the factorization
        return factors;
    }

    /// # Summary
    /// Tries to find the factors of `modulus` given a `period` and `generator`.
    ///
    /// # Input
    /// ## modulus
    /// The modulus which defines the residue ring Z mod `modulus` in which the
    /// multiplicative order of `generator` is being estimated.
    /// ## generator
    /// The unsigned integer multiplicative order (period) of which is being
    /// estimated. Must be co-prime to `modulus`.
    /// ## period
    /// The estimated period (multiplicative order) of the generator mod
    /// `modulus`.
    ///
    /// # Output
    /// A tuple of a flag indicating whether factors were found successfully,
    /// and a pair of integers representing the factors that were found.
    /// Note that the second output is only meaningful when the first output is
    /// `true`.
    function MaybeFactorsFromPeriod(
        modulus : Int,
        generator : Int,
        period : Int
    ) : (Bool, (Int, Int)) {

        // Period finding reduces to factoring only if period is even
        if period % 2 == 0 {
            // Compute `generator` ^ `period/2` mod `number`.
            let halfPower = ExpModI(generator, period / 2, modulus);

            // If we are unlucky, halfPower is just -1 mod N, which is a trivial
            // case and not useful for factoring.
            if halfPower != modulus - 1 {
                // When the halfPower is not -1 mod N, halfPower-1 or
                // halfPower+1 share non-trivial divisor with `number`. Find it.
                let factor = MaxI(
                    GreatestCommonDivisorI(halfPower - 1, modulus),
                    GreatestCommonDivisorI(halfPower + 1, modulus)
                );

                // Add a flag that we found the factors, and return only if computed
                // non-trivial factors (not like 1:n or n:1)
                if (factor != 1) and (factor != modulus) {
                    Message($"Found factor={factor}");
                    return (true, (factor, modulus / factor));
                }
            }
            // Return a flag indicating we hit a trivial case and didn't get
            // any factors.
            Message($"Found trivial factors.");
            return (false, (1, 1));
        } else {
            // When period is odd we have to pick another generator to estimate
            // period of and start over.
            Message($"Estimated period {period} was odd, trying again.");
            return (false, (1, 1));
        }
    }

    /// # Summary
    /// Find the period of a number from an input frequency.
    ///
    /// # Input
    /// ## modulus
    /// The modulus which defines the residue ring Z mod `modulus` in which the
    /// multiplicative order of `generator` is being estimated.
    /// ## frequencyEstimate
    /// The frequency that we want to convert to a period.
    /// ## bitsPrecision
    /// Number of bits of precision with which we need to estimate s/r to
    /// recover period r using continued fractions algorithm.
    /// ## currentDivisor
    /// The divisor of the generator period found so far.
    ///
    /// # Output
    /// The period as calculated from the estimated frequency via the continued
    /// fractions algorithm.
    function PeriodFromFrequency(
        modulus : Int,
        frequencyEstimate : Int,
        bitsPrecision : Int,
        currentDivisor : Int
    ) : Int {
        // Now we use the ContinuedFractionConvergentI function to recover s/r
        // from dyadic fraction k/2^bitsPrecision.
        let (numerator, period) = ContinuedFractionConvergentI(
            (frequencyEstimate, 2^bitsPrecision),
            modulus
        );

        // ContinuedFractionConvergentI does not guarantee the signs of the
        // numerator and denominator. Here we make sure that both are positive
        // using AbsI.
        let (numeratorAbs, periodAbs) = (AbsI(numerator), AbsI(period));

        // Compute and return the newly found divisor.
        let period = (periodAbs * currentDivisor) / GreatestCommonDivisorI(currentDivisor, periodAbs);
        Message($"Found period={period}");
        return period;
    }

    /// # Summary
    /// Finds a multiplicative order of the generator in the residue ring Z mod
    /// `modulus`.
    ///
    /// # Input
    /// ## generator
    /// The unsigned integer multiplicative order (period) of which is being
    /// estimated. Must be co-prime to `modulus`.
    /// ## modulus
    /// The modulus which defines the residue ring Z mod `modulus` in which the
    /// multiplicative order of `generator` is being estimated.
    ///
    /// # Output
    /// The period (multiplicative order) of the generator mod `modulus`
    operation EstimatePeriod(generator : Int, modulus : Int) : Int {
        // Here we check that the inputs to the EstimatePeriod operation are
        // valid.
        Fact(
            GreatestCommonDivisorI(generator, modulus) == 1,
            "`generator` and `modulus` must be co-prime"
        );

        // Number of bits in the modulus with respect to which we are estimating
        // the period.
        let bitsize = BitSizeI(modulus);

        // The EstimatePeriod operation estimates the period r by finding an
        // approximation k/2^(bits precision) to a fraction s/r, where s is some
        // integer. Note that if s and r have common divisors we will end up
        // recovering a divisor of r and not r itself.

        // Number of bits of precision with which we need to estimate s/r to
        // recover period r, using continued fractions algorithm.
        let bitsPrecision = 2 * bitsize + 1;

        // Current estimate for the frequency of the form s/r.
        let frequencyEstimate = EstimateFrequency(generator, modulus, bitsize);
        if frequencyEstimate != 0 {
            return PeriodFromFrequency(
                modulus,
                frequencyEstimate,
                bitsPrecision,
                1
            );
        } else {
            Message("The estimated frequency was 0, trying again.");
            return 1;
        }
    }

    /// # Summary
    /// Estimates the frequency of a generator in the residue ring Z mod
    /// `modulus`.
    ///
    /// # Input
    /// ## generator
    /// The unsigned integer multiplicative order (period) of which is being
    /// estimated. Must be co-prime to `modulus`.
    /// ## modulus
    /// The modulus which defines the residue ring Z mod `modulus` in which the
    /// multiplicative order of `generator` is being estimated.
    /// ## bitsize
    /// Number of bits needed to represent the modulus.
    ///
    /// # Output
    /// The numerator k of dyadic fraction k/2^bitsPrecision approximating s/r.
    operation EstimateFrequency(generator : Int, modulus : Int, bitsize : Int) : Int {
        mutable frequencyEstimate = 0;
        let bitsPrecision = 2 * bitsize + 1;
        Message($"Estimating frequency with bitsPrecision={bitsPrecision}.");

        // Allocate qubits for the superposition of eigenstates of the oracle
        // that is used in period finding.
        use eigenstateRegister = Qubit[bitsize];

        // Initialize eigenstateRegister to 1, which is a superposition of the
        // eigenstates we are estimating the phases of.
        // We are interpreting the register as encoding an unsigned integer in
        // little-endian format.
        ApplyXorInPlace(1, eigenstateRegister);

        // Use phase estimation with a semiclassical Fourier transform to
        // estimate the frequency.
        use c = Qubit();
        for idx in bitsPrecision - 1..-1..0 {
            H(c);
            Controlled ApplyOrderFindingOracle(
                [c],
                (generator, modulus, 1 <<< idx, eigenstateRegister)
            );
            R1Frac(frequencyEstimate, bitsPrecision - 1 - idx, c);
            H(c);
            if M(c) == One {
                X(c); // Reset
                set frequencyEstimate += 1 <<< (bitsPrecision - 1 - idx);
            }
        }

        // Return all the qubits used for oracle's eigenstate back to 0 state
        // using ResetAll.
        ResetAll(eigenstateRegister);
        Message($"Estimated frequency={frequencyEstimate}");
        return frequencyEstimate;
    }

    /// # Summary
    /// Interprets `target` as encoding unsigned little-endian integer k
    /// and performs transformation |k⟩ ↦ |gᵖ⋅k mod N ⟩ where
    /// p is `power`, g is `generator` and N is `modulus`.
    ///
    /// # Input
    /// ## generator
    /// The unsigned integer multiplicative order (period)
    /// of which is being estimated. Must be co-prime to `modulus`.
    /// ## modulus
    /// The modulus which defines the residue ring Z mod `modulus`
    /// in which the multiplicative order of `generator` is being estimated.
    /// ## power
    /// Power of `generator` by which `target` is multiplied.
    /// ## target
    /// Register interpreted as little-endian which is multiplied by
    /// given power of the generator. The multiplication is performed modulo
    /// `modulus`.
    internal operation ApplyOrderFindingOracle(
        generator : Int,
        modulus : Int,
        power : Int,
        target : Qubit[]
    ) : Unit is Adj + Ctl {
        // The oracle we use for order finding implements |x⟩ ↦ |x⋅a mod N⟩. We
        // also use `ExpModI` to compute a by which x must be multiplied. Also
        // note that we interpret target as unsigned integer in little-endian
        // format.
        ModularMultiplyByConstant(
            modulus,
            ExpModI(generator, power, modulus),
            target
        );
    }

    /// # Summary
    /// Performs modular in-place multiplication by a classical constant.
    ///
    /// # Description
    /// Given the classical constants `c` and `modulus`, and an input quantum
    /// register |𝑦⟩ in little-endian format, this operation computes
    /// `(c*x) % modulus` into |𝑦⟩.
    ///
    /// # Input
    /// ## modulus
    /// Modulus to use for modular multiplication
    /// ## c
    /// Constant by which to multiply |𝑦⟩
    /// ## y
    /// Quantum register of target
    internal operation ModularMultiplyByConstant(modulus : Int, c : Int, y : Qubit[]) : Unit is Adj + Ctl {
        use qs = Qubit[Length(y)];
        for idx in IndexRange(y) {
            let shiftedC = (c <<< idx) % modulus;
            Controlled ModularAddConstant(
                [y[idx]],
                (modulus, shiftedC, qs)
            );
        }
        for idx in IndexRange(y) {
            SWAP(y[idx], qs[idx]);
        }
        let invC = InverseModI(c, modulus);
        for idx in IndexRange(y) {
            let shiftedC = (invC <<< idx) % modulus;
            Controlled ModularAddConstant(
                [y[idx]],
                (modulus, modulus - shiftedC, qs)
            );
        }
    }

    /// # Summary
    /// Performs modular in-place addition of a classical constant into a
    /// quantum register.
    ///
    /// Given the classical constants `c` and `modulus`, and an input quantum
    /// register |𝑦⟩ in little-endian format, this operation computes
    /// `(x+c) % modulus` into |𝑦⟩.
    ///
    /// # Input
    /// ## modulus
    /// Modulus to use for modular addition
    /// ## c
    /// Constant to add to |𝑦⟩
    /// ## y
    /// Quantum register of target
    internal operation ModularAddConstant(modulus : Int, c : Int, y : Qubit[]) : Unit is Adj + Ctl {
        body (...) {
            Controlled ModularAddConstant([], (modulus, c, y));
        }
        controlled (ctrls, ...) {
            // We apply a custom strategy to control this operation instead of
            // letting the compiler create the controlled variant for us in
            // which the `Controlled` functor would be distributed over each
            // operation in the body.
            //
            // Here we can use some scratch memory to save ensure that at most
            // one control qubit is used for costly operations such as
            // `AddConstant` and `CompareGreaterThenOrEqualConstant`.
            if Length(ctrls) >= 2 {
                use control = Qubit();
                within {
                    Controlled X(ctrls, control);
                } apply {
                    Controlled ModularAddConstant([control], (modulus, c, y));
                }
            } else {
                use carry = Qubit();
                Controlled IncByI(ctrls, (c, y + [carry]));
                Controlled Adjoint IncByI(ctrls, (modulus, y + [carry]));
                Controlled IncByI([carry], (modulus, y));
                Controlled ApplyIfLessOrEqualL(ctrls, (X, IntAsBigInt(c), y, carry));
            }
        }
    }

    // -- library code from `Unstable` below --

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

    operation ApplyOrAssuming0Target(control1 : Qubit, control2 : Qubit, target : Qubit) : Unit is Adj {
        within {
            X(control1);
            X(control2);
        } apply {
            ApplyAndAssuming0Target(control1, control2, target);
            X(target);
        }
    }

}
