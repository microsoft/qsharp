/// # Sample
/// Shor's algorithm
///
/// # Description
/// Shor's algorithm is a quantum algorithm for finding the prime factors of an
/// integer.
///
/// This Q# program implements Shor's algorithm.
namespace Sample {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Random;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Arithmetic;
    open Microsoft.Quantum.Arrays;

    @EntryPoint()
    operation Main() : (Int, Int) {
        let n = 143; // 11*13;
        // You can try these other examples for a lengthier computation.
        // let n = 16837; // = 113*149
        // let n = 22499; // = 149*151

        // Use Shor's algorithm to factor a semiprime integer.
        let (a, b) = FactorSemiprimeInteger(n);
        Message($"Found factorization {n} = {a} * {b} ");
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
                set (foundFactors, factors) =
                    MaybeFactorsFromPeriod(number, generator, period);
            }
            // In this case, we guessed a divisor by accident.
            else {
                // Find divisor.
                let gcd = GreatestCommonDivisorI(number, generator);
                Message($"We have guessed a divisor {gcd} by accident. " +
                    "No quantum computation was done.");

                // Set the flag `foundFactors` to true, indicating that we
                // succeeded in finding factors.
                set foundFactors = true;
                set factors = (gcd, number / gcd);
            }
            set attempt = attempt+1;
            if (attempt > 100) {
                fail "Failed to find factors: too many attempts!";
            }
        }
        until foundFactors
        fixup {
            Message("The estimated period did not yield a valid factor. " +
                "Trying again.");
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
        period : Int)
    : (Bool, (Int, Int)) {

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
        currentDivisor : Int)
    : Int {
        // Now we use the ContinuedFractionConvergentI function to recover s/r
        // from dyadic fraction k/2^bitsPrecision.
        let (numerator, period) = ContinuedFractionConvergentI(
            (frequencyEstimate, 2 ^ bitsPrecision),
            modulus);

        // ContinuedFractionConvergentI does not guarantee the signs of the
        // numerator and denominator. Here we make sure that both are positive
        // using AbsI.
        let (numeratorAbs, periodAbs) = (AbsI(numerator), AbsI(period));

        // Compute and return the newly found divisor.
        let period =
            (periodAbs * currentDivisor) /
            GreatestCommonDivisorI(currentDivisor, periodAbs);
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
            "`generator` and `modulus` must be co-prime");

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
                modulus, frequencyEstimate, bitsPrecision, 1);
        }
        else {
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
    operation EstimateFrequency(generator : Int,modulus : Int, bitsize : Int)
    : Int {
        mutable frequencyEstimate = 0;
        let bitsPrecision =  2 * bitsize + 1;
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
        for idx in bitsPrecision-1..-1..0 {
            H(c);
            Controlled ApplyOrderFindingOracle(
                [c],
                (generator, modulus, 1 <<< idx, eigenstateRegister));
            R1Frac(frequencyEstimate, bitsPrecision-1-idx, c);
            H(c);
            if M(c) == One {
                X(c); // Reset
                set frequencyEstimate += 1 <<< (bitsPrecision-1-idx);
            }
        }

        // Return all the qubits used for oracle's eigenstate back to 0 state
        // using ResetAll.
        ResetAll(eigenstateRegister);
        Message($"Estimated frequency={frequencyEstimate}");
        return frequencyEstimate;
    }

    /// # Summary
    /// Interprets `target` as encoding unsigned little-endian integer k and
    /// performs transformation |k⟩ ↦ |gᵖ⋅k mod N ⟩ where p is `power`, g is
    /// `generator` and N is `modulus`.
    ///
    /// # Input
    /// ## generator
    /// The unsigned integer multiplicative order (period) of which is being
    /// estimated. Must be co-prime to `modulus`.
    /// ## modulus
    /// The modulus which defines the residue ring Z mod `modulus` in which the
    /// multiplicative order of `generator` is being estimated.
    /// ## power
    /// Power of `generator` by which `target` is multiplied.
    /// ## target
    /// Register interpreted as little-endian which is multiplied by given power
    /// of the generator. The multiplication is performed modulo `modulus`.
    operation ApplyOrderFindingOracle(
        generator : Int,
        modulus : Int,
        power : Int,
        target : Qubit[])
    : Unit is Adj + Ctl {
        // Check that the parameters satisfy the requirements.
        Fact(
            GreatestCommonDivisorI(generator, modulus) == 1,
            "`generator` and `modulus` must be co-prime");

        // The oracle we use for order finding implements |x⟩ ↦ |x⋅a mod N⟩.
        // We also use `ExpModI` to compute a by which x must be multiplied.
        // Also note that we interpret target as unsigned integer in
        // little-endian fromat.
        ModularMultiplyByConstant(
            modulus,
            ExpModI(generator, power, modulus),
            target);
    }

    //
    // Arithmetic helper functions to implement order-finding oracle.
    //

    /// # Summary
    /// Returns the number of trailing zeroes of a number.
    ///
    /// ## Example
    /// let zeroes = NTrailingZeroes(21); // NTrailingZeroes(0b1101) = 0
    /// let zeroes = NTrailingZeroes(20); // NTrailingZeroes(0b1100) = 2
    function NTrailingZeroes(number : Int) : Int {
        Fact(number != 0, "NTrailingZeroes: number cannot be 0.");
        mutable nZeroes = 0;
        mutable copy = number;
        while (copy % 2 == 0) {
            set nZeroes += 1;
            set copy /= 2;
        }
        return nZeroes;
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
    operation ModularMultiplyByConstant(modulus : Int, c : Int, y : Qubit[])
    : Unit is Adj + Ctl {
        use qs = Qubit[Length(y)];
        for idx in IndexRange(y) {
            let shiftedC = (c <<< idx) % modulus;
            Controlled ModularAddConstant(
                [y[idx]],
                (modulus, shiftedC, qs));
        }
        for idx in IndexRange(y) {
            SWAP(y[idx], qs[idx]);
        }
        let invC = InverseModI(c, modulus);
        for idx in IndexRange(y) {
            let shiftedC = (invC <<< idx) % modulus;
            Controlled ModularAddConstant(
                [y[idx]],
                (modulus, modulus - shiftedC, qs));
        }
    }

    /// # Summary
    /// Performs modular in-place addition of a classical constant into a
    /// quantum register.
    ///
    /// # Description
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
    operation ModularAddConstant(modulus : Int, c : Int, y : Qubit[])
    : Unit is Adj + Ctl {
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
                    Controlled ModularAddConstant(
                        [control],
                        (modulus, c, y));
                }
            } else {
                use carry = Qubit();
                Controlled AddConstant(
                    ctrls, (c, y + [carry]));
                Controlled Adjoint AddConstant(
                    ctrls, (modulus, y + [carry]));
                Controlled AddConstant(
                    [carry], (modulus, y));
                Controlled CompareGreaterThanOrEqualConstant(
                    ctrls, (c, y, carry));
            }
        }
    }

    /// # Summary
    /// Performs in-place addition of a constant into a quantum register.
    ///
    /// # Description
    /// Given a non-empty quantum register |𝑦⟩ of length 𝑛+1 and a positive
    // constant 𝑐 < 2ⁿ, computes |𝑦 + c⟩ into |𝑦⟩.
    ///
    /// # Input
    /// ## c
    /// Constant number to add to |𝑦⟩.
    /// ## y
    /// Quantum register of second summand and target; must not be empty.
    operation AddConstant(c : Int, y : Qubit[]) : Unit is Adj + Ctl {
        // We are using this version instead of the library version that is
        // based on Fourier angles to show an advantage of sparse simulation
        // in this sample.
        let n = Length(y);
        Fact(n > 0, "Bit width must be at least 1");
        Fact(c >= 0, "constant must not be negative");
        Fact(c < 2^n, "constant must be smaller than {2^n)}");
        if c != 0 {
            // If c has j trailing zeroes than the j least significant bits of y
            // won't be affected by the addition and can therefore be ignored by
            // applying the addition only to the other qubits and shifting c
            // accordingly.
            let j = NTrailingZeroes(c);
            use x = Qubit[n - j];
            within {
                ApplyXorInPlace(c >>> j, x);
            } apply {
                AddI(x, y[j...]);
            }
        }
    }

    /// # Summary
    /// Performs greater-than-or-equals comparison to a constant.
    ///
    /// # Description
    /// Toggles output qubit `target` if and only if input register `x` is
    /// greater than or equal to `c`.
    ///
    /// # Input
    /// ## c
    /// Constant value for comparison.
    /// ## x
    /// Quantum register to compare against.
    /// ## target
    /// Target qubit for comparison result.
    ///
    /// # Reference
    /// This construction is described in [Lemma 3, arXiv:2201.10200]
    operation CompareGreaterThanOrEqualConstant(
        c : Int,
        x : Qubit[],
        target : Qubit)
    : Unit is Adj+Ctl {
        let bitWidth = Length(x);
        if c == 0 {
            X(target);
        } elif c >= (2^bitWidth) {
            // do nothing
        } elif c == (2^(bitWidth - 1)) {
            CNOT(Tail(x), target);
        } else {
            // normalize constant
            let l = NTrailingZeroes(c);
            let cNormalized = c >>> l;
            let xNormalized = x[l...];
            let bitWidthNormalized = Length(xNormalized);
            use qs = Qubit[bitWidthNormalized - 1];
            let cs1 = [Head(xNormalized)] + Most(qs);
            Fact(Length(cs1) == Length(qs),
                "Arrays should be of the same length.");
            within {
                for i in 0..Length(cs1)-1 {
                    let op =
                        cNormalized &&& (1 <<< (i+1)) != 0 ?
                        ApplyAnd | ApplyOr;
                    op(cs1[i], xNormalized[i+1], qs[i]);
                }
            } apply {
                CNOT(Tail(qs), target);
            }
        }
    }

    /// # Summary
    /// Inverts a given target qubit if and only if both control qubits are in
    /// the 1 state, using measurement to perform the adjoint operation.
    ///
    /// # Description
    /// Inverts `target` if and only if both controls are 1, but assumes that
    /// `target` is in state 0. The operation has T-count 4, T-depth 2 and
    /// requires no helper qubit, and may therefore be preferable to a CCNOT
    /// operation, if `target` is known to be 0.
    /// The adjoint of this operation is measurement based and requires no T
    /// gates.
    /// Although the Toffoli gate (CCNOT) will perform faster in in simulations,
    /// this version has lower T gate requirements.
    ///
    /// # Input
    /// ## control1
    /// First control qubit
    /// ## control2
    /// Second control qubit
    /// ## target
    /// Target auxiliary qubit; must be in state 0
    ///
    /// # References
    /// - Cody Jones: "Novel constructions for the fault-tolerant
    ///   Toffoli gate",
    ///   Phys. Rev. A 87, 022328, 2013
    ///   [arXiv:1212.5069](https://arxiv.org/abs/1212.5069)
    ///   doi:10.1103/PhysRevA.87.022328
    /// - Craig Gidney: "Halving the cost of quantum addition",
    ///   Quantum 2, page 74, 2018
    ///   [arXiv:1709.06648](https://arxiv.org/abs/1709.06648)
    ///   doi:10.1103/PhysRevA.85.044302
    /// - Mathias Soeken: "Quantum Oracle Circuits and the Christmas
    ///   Tree Pattern",
    ///   [Blog article from December 19, 2019](https://msoeken.github.io/blog_qac.html)
    ///   (note: explains the multiple controlled construction)
    operation ApplyAnd(control1 : Qubit, control2 : Qubit, target : Qubit)
    : Unit is Adj {
        body (...) {
            H(target);
            T(target);
            CNOT(control1, target);
            CNOT(control2, target);
            within {
                CNOT(target, control1);
                CNOT(target, control2);
            }
            apply {
                Adjoint T(control1);
                Adjoint T(control2);
                T(target);
            }
            H(target);
            S(target);
        }
        adjoint (...) {
            H(target);
            if (M(target) == One) {
                X(target);
                CZ(control1, control2);
            }
        }
    }

    /// # Summary
    /// Applies X to the target if any of the controls are 1.
    operation ApplyOr(control1 : Qubit, control2 : Qubit, target : Qubit)
    : Unit is Adj {
        within {
            ApplyToEachA(X, [control1, control2]);
        } apply {
            ApplyAnd(control1, control2, target);
            X(target);
        }
    }
}
