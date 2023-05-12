namespace Microsoft.Quantum.Samples.Shor {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Random;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Canon;
    open Microsoft.Quantum.Arithmetic;
    open Microsoft.Quantum.Arrays;

    function MaybeFactorsFromPeriod(modulus : Int, generator : Int, period : Int)
    : (Bool, (Int, Int)) {
        // Period finding reduces to factoring only if period is even
        if period % 2 == 0 {
            // Compute `generator` ^ `period/2` mod `number`
            // using Microsoft.Quantum.Math.ExpModI.
            let halfPower = ExpModI(generator, period / 2, modulus);

            // If we are unlucky, halfPower is just -1 mod N,
            // which is a trivial case and not useful for factoring.
            if halfPower != modulus - 1 {
                // When the halfPower is not -1 mod N
                // halfPower-1 or halfPower+1 share non-trivial divisor with `number`.
                // We find a divisor Microsoft.Quantum.Math.GreatestCommonDivisorI.
                let factor = MaxI(
                    GreatestCommonDivisorI(halfPower - 1, modulus),
                    GreatestCommonDivisorI(halfPower + 1, modulus)
                );

                // Add a flag that we found the factors, and return computed non-trivial factors.
                return (true, (factor, modulus / factor));
            } else {
                // Return a flag indicating we hit a trivial case and didn't get any factors.
                return (false, (1, 1));
            }
        } else {
            // When period is odd we have to pick another generator to estimate
            // period of and start over.
            Message($"Estimated period {period} was odd, trying again.");
            return (false, (1, 1));
        }
    }

    function PeriodFromFrequency(
        modulus : Int,
        frequencyEstimate : Int,
        bitsPrecision : Int,
        currentDivisor : Int
    )
    : Int {
        Message($"modulus={modulus}, frequencyEstimate={frequencyEstimate}, bitsPrecision={bitsPrecision}, currentDivisor={currentDivisor}");

        // Now we use Microsoft.Quantum.Math.ContinuedFractionConvergentI
        // function to recover s/r from dyadic fraction k/2^bitsPrecision.
        let (numerator, period) = (ContinuedFractionConvergentI((frequencyEstimate, 2 ^ bitsPrecision), modulus));
        Message($"{frequencyEstimate} / {2^bitsPrecision} ~= {AbsI(numerator)}/{AbsI(period)} mod {modulus}");

        // ContinuedFractionConvergentI does not guarantee the signs of the numerator
        // and denominator. Here we make sure that both are positive using
        // AbsI.
        let (numeratorAbs, periodAbs) = (AbsI(numerator), AbsI(period));

        // Return the newly found divisor.
        // Uses Microsoft.Quantum.Math.GreatestCommonDivisorI function from Microsoft.Quantum.Math.
        return (periodAbs * currentDivisor) / GreatestCommonDivisorI(currentDivisor, periodAbs);
    }
    
    operation EstimatePeriod(
        generator : Int, modulus : Int
    )
    : Int {
        // Here we check that the inputs to the EstimatePeriod operation are valid.
        Fact(GreatestCommonDivisorI(generator, modulus) == 1, "`generator` and `modulus` must be co-prime");

        // The variable that stores the divisor of the generator period found so far.
        mutable result = 1;

        // Number of bits in the modulus with respect to which we are estimating the period.
        let bitsize = BitSizeI(modulus);

        // The EstimatePeriod operation estimates the period r by finding an
        // approximation k/2^(bits precision) to a fraction s/r, where s is some integer.
        // Note that if s and r have common divisors we will end up recovering a divisor of r
        // and not r itself. However, if we recover enough divisors of r
        // we recover r itself pretty soon.

        // Number of bits of precision with which we need to estimate s/r to recover period r.
        // using continued fractions algorithm.
        let bitsPrecision = 2 * bitsize + 1;

        // A variable that stores our current estimate for the frequency
        // of the form s/r.
        mutable frequencyEstimate = 0;

        set frequencyEstimate = EstimateFrequency(
            generator, modulus, bitsize
        );

        if frequencyEstimate != 0 {
            set result = PeriodFromFrequency(modulus, frequencyEstimate, bitsPrecision, result);
        }
        else {
            Message("The estimated frequency was 0, trying again.");
        }
        return result;
    }

    operation FactorSemiprimeInteger(number : Int)
    : (Int, Int) {
        // First check the most trivial case, if the provided number is even
        if number % 2 == 0 {
            Message("An even number has been given; 2 is a factor.");
            return (number / 2, 2);
        }
        // These mutables will keep track of if we found the factors,
        // and if so, what they are. The default value for the factors
        // is (1,1).
        mutable foundFactors = false;
        mutable factors = (1, 1);
        mutable attempt = 1;

        repeat {
            // Next try to guess a number co-prime to `number`
            // Get a random integer in the interval [1,number-1]
            let generator = DrawRandomInt(1, number - 1);

            Message($"******* Attempt {attempt} with generator={generator}, number={number}");

            // Check if the random integer indeed co-prime using
            // Microsoft.Quantum.Math.IsCoprimeI.
            // If true use Quantum algorithm for Period finding.
            if GreatestCommonDivisorI(generator, number) == 1 {
                // Print a message using Microsoft.Quantum.Intrinsic.Message
                // indicating that we are doing something quantum.
                Message($"Estimating period of {generator}");

                // Call Quantum Period finding algorithm for
                // `generator` mod `number`.
                let period = EstimatePeriod(generator, number);

                // Set the flag and factors values if the continued fractions
                // classical algorithm succeeds.
                set (foundFactors, factors) = MaybeFactorsFromPeriod(number, generator, period);
            }
            // In this case, we guessed a divisor by accident.
            else {
                // Find a divisor using Microsoft.Quantum.Math.GreatestCommonDivisorI
                let gcd = GreatestCommonDivisorI(number, generator);

                // Don't forget to tell the user that we were lucky and didn't do anything
                // quantum by using Microsoft.Quantum.Intrinsic.Message.
                Message($"We have guessed a divisor of {number} to be {gcd} by accident.");

                // Set the flag `foundFactors` to true, indicating that we succeeded in finding
                // factors.
                set foundFactors = true;
                set factors = (gcd, number / gcd);
            }
            set attempt = attempt+1;
            if (attempt > 100) {
                fail "Too many attempts!";
            }
        }
        until foundFactors
        fixup {
            Message("The estimated period did not yield a valid factor, trying again.");
        }

        // Return the factorization
        return factors;
    }

    // -------------------------------------------------
    // Quantum part
    //

    operation EstimateFrequency(
        generator : Int,
        modulus : Int,
        bitsize : Int
    )
    : Int {
        mutable frequencyEstimate = 0;
        let bitsPrecision =  2 * bitsize + 1;

        // Allocate qubits for the superposition of eigenstates of
        // the oracle that is used in period finding.
        use eigenstateRegister = Qubit[bitsize];

        // Initialize eigenstateRegister to 1, which is a superposition of
        // the eigenstates we are estimating the phases of.
        // We first interpret the register as encoding an unsigned integer
        // in little endian encoding.
        // let eigenstateRegisterLE = LittleEndian(eigenstateRegister);
        ApplyXorInPlace(1, eigenstateRegister);
        // let oracle = ApplyOrderFindingOracle(generator, modulus, _, _);

        // Use phase estimation with a semiclassical Fourier transform to
        // estimate the frequency.
        use c = Qubit();
        for idx in bitsPrecision - 1..-1..0 {
            H(c);
            Controlled ApplyOrderFindingOracle([c], (generator, modulus, 1 <<< idx, eigenstateRegister));
            R1Frac(frequencyEstimate, bitsPrecision - 1 - idx, c);
            H(c);
            if M(c) == One {
                X(c); // Reset
                set frequencyEstimate += 1 <<< (bitsPrecision - 1 - idx);
            }
        }

        // Return all the qubits used for oracle's eigenstate back to 0 state
        // using Microsoft.Quantum.Intrinsic.ResetAll.
        ResetAll(eigenstateRegister);

        return frequencyEstimate;
    }

    internal operation ApplyOrderFindingOracle(
        generator : Int, modulus : Int, power : Int, target : Qubit[]
    ) : Unit is Adj + Ctl {
        // Check that the parameters satisfy the requirements.
        Fact(GreatestCommonDivisorI(generator, modulus) == 1, "`generator` and `modulus` must be co-prime");

        // The oracle we use for order finding implements |x⟩ ↦ |x⋅a mod N⟩. We
        // also use `ExpModI` to compute a by which x must be multiplied. Also
        // note that we interpret target as unsigned integer in little-endian
        // encoding by using the `LittleEndian` type.
        ModularMultiplyByConstant(modulus,
                                  ExpModI(generator, power, modulus),
                                  target);
    }

    internal operation ModularMultiplyByConstant(modulus : Int, c : Int, y : Qubit[])
    : Unit is Adj + Ctl {
        use qs = Qubit[Length(y)];
        for idx in IndexRange(y) {
            let shiftedC = (c <<< idx) % modulus;
            Controlled ModularAddConstant([y[idx]], (modulus, shiftedC, qs));
        }
        for idx in IndexRange(y) {
            SWAP(y[idx], qs[idx]);
        }
        let invC = InverseModI(c, modulus);
        for idx in IndexRange(y) {
            let shiftedC = (invC <<< idx) % modulus;
            Controlled ModularAddConstant([y[idx]], (modulus, modulus - shiftedC, qs));
        }
    }

    // NOTE: MOSTLY TESTED
    internal operation ModularAddConstant(modulus : Int, c : Int, y : Qubit[])
    : Unit is Adj + Ctl {
        body (...) {
            Controlled ModularAddConstant([], (modulus, c, y));
        }
        controlled (ctrls, ...) {
            // We apply a custom strategy to control this operation instead of
            // letting the compiler create the controlled variant for us in which
            // the `Controlled` functor would be distributed over each operation
            // in the body.
            //
            // Here we can use some scratch memory to save ensure that at most one
            // control qubit is used for costly operations such as `AddConstant`
            // and `CompareGreaterThenOrEqualConstant`.
            if Length(ctrls) >= 2 {
                use control = Qubit();
                within {
                    Controlled X(ctrls, control);
                } apply {
                    Controlled ModularAddConstant([control], (modulus, c, y));
                }
            } else {
                use carry = Qubit();
                Controlled AddConstant(ctrls, (c, y + [carry]));
                Controlled Adjoint AddConstant(ctrls, (modulus, y + [carry]));
                Controlled AddConstant([carry], (modulus, y));
                Controlled CompareGreaterThanOrEqualConstant(ctrls, (c, y, carry));
            }
        }
    }

    // NOTE: TESTED
    internal operation AddConstant(c : Int, y : Qubit[]) : Unit is Adj + Ctl {
        // We are using this version instead of the library version that is based
        // on Fourier angles to show an advantage of sparse simulation in this sample.

        let n = Length(y);
        Fact(n > 0, "Bit width must be at least 1");

        Fact(c >= 0, "constant must not be negative");
        Fact(c < 2^n, "constant must be smaller than {2^n)}");

        if c != 0 {
            // If c has j trailing zeroes than the j least significant bits
            // of y won't be affected by the addition and can therefore be
            // ignored by applying the addition only to the other qubits and
            // shifting c accordingly.
            let j = NTrailingZeroes(c);
            use x = Qubit[n - j];
            within {
                ApplyXorInPlace(c >>> j, x);
            } apply {
                AddI(x, y[j...]);
            }
        }
    }

    // NOTE: TESTED
    internal operation CompareGreaterThanOrEqualConstant(c : Int, x : Qubit[], target : Qubit)
    : Unit is Adj+Ctl {
        let bitWidth = Length(x);

        if c == 0 {
            X(target);
        } elif c >= (2^bitWidth) {
            // do nothing
        } elif c == (2^(bitWidth - 1)) {
            ApplyLowTCNOT(Tail(x), target);
        } else {
            // normalize constant
            let l = NTrailingZeroes(c);

            let cNormalized = c >>> l;
            let xNormalized = x[l...];
            let bitWidthNormalized = Length(xNormalized);

            use qs = Qubit[bitWidthNormalized - 1];
            let cs1 = [Head(xNormalized)] + Most(qs);

            Fact(cs1::Length == qs::Length, "Arrays should be of the same length");

            within {
                for i in 0..cs1::Length-1 {
                    ((cNormalized &&& 2^(i+1)) != 0 ? ApplyAnd | ApplyOr)(cs1[i], xNormalized[i+1], qs[i]);
                }
            } apply {
                ApplyLowTCNOT(Tail(qs), target);
            }
        }
    }

    // NOTE: TESTED
    internal operation ApplyOr(control1 : Qubit, control2 : Qubit, target : Qubit) : Unit is Adj+Ctl {
        within {
            ApplyToEachA(X, [control1, control2]);
        } apply {
            ApplyAnd(control1, control2, target);
            X(target);
        }
    }

    // NOTE: TESTED
    // TODO: Check if a different version of ApplyAnd will be faster...
    internal operation ApplyAnd(control1: Qubit, control2: Qubit, target: Qubit): Unit is Adj+Ctl {
        CCNOT(control1, control2, target);
    }

    // NOTE: TESTED
    internal function NTrailingZeroes(number : Int) : Int {
        Fact(number != 0, "NTrailingZeroes doesn't work with 0 argument");
        mutable nZeroes = 0;
        mutable copy = number;
        while (copy % 2 == 0) {
            set nZeroes += 1;
            set copy /= 2;
        }
        return nZeroes;
    }

    // NOTE: TESTED
    internal operation ApplyLowTCNOT(a : Qubit, b : Qubit) : Unit is Adj+Ctl {
        body (...) {
            CNOT(a, b);
        }

        adjoint self;

        controlled (ctls, ...) {
            // In this application this operation is used in a way that
            // it is controlled by at most one qubit.
            Fact(Length(ctls) <= 1, "At most one control line allowed");

            if ctls::Length == 0 {
                CNOT(a, b);
            } else {
                use q = Qubit();
                within {
                    ApplyAnd(Head(ctls), a, q);
                } apply {
                    CNOT(q, b);
                }
            }
        }

        controlled adjoint self;
    }


    // operation EstimateFrequency(
    //     generator : Int,
    //     modulus : Int,
    //     bitsize : Int
    // )
    // : Int {
    //     DrawRandomInt(0, modulus)
    // }


    //
    // Quantum part
    // -------------------------------------------------

    @EntryPoint()
    operation Main() : Result[] {
        Message("Starting...");

        // let n = 263*373;
        // let n = 17017;
        let n = 16837;
        let (a, b) = FactorSemiprimeInteger(n);
        Message($"{n} = {a} * {b} ");

        // use q = Qubit[3];
        // for i in 0..7 {
        //     ApplyXorInPlace(i,q);
        //     DumpMachine();
        //     Adjoint Controlled ApplyLowTCNOT([q[2]],(q[1],q[0]));
        //     DumpMachine();
        //     ResetAll(q);
        // }

        //for i in 1..8 {
        //    Message(AsString(i) + " -> " + AsString(NTrailingZeroes(i)));
        //}

        // // CompareGreaterThanOrEqualConstant test
        // use q = Qubit[3];
        // use t = Qubit();
        // for clas in 0..10 {
        //     for quan in 0..7 {
        //         ApplyXorInPlace(quan,q);
        //         CompareGreaterThanOrEqualConstant(clas, q, t);
        //         Message( AsString(quan) + " >= " + AsString(clas) + ": " + AsString(M(t)));
        //         Fact( (quan>=clas) == (M(t) == One), "Incorrect!" );
        //         //DumpMachine();
        //         ResetAll(q+[t]);
        //     }
        // }
        // Message("Correct.");

        // use q = Qubit[3];
        // for clas in 0..10 {
        //     for quan in 0..7 {
        //         ApplyXorInPlace(quan,q);
        //         AddConstant(clas, q);
        //         Message( AsString(clas) + " + " + AsString(quan) + "= " + AsString([M(q[2]),M(q[1]),M(q[0])]));
        //         //Fact( (quan>=clas) == (M(t) == One), "Incorrect!" );
        //         //DumpMachine();
        //         ResetAll(q);
        //     }
        // }
        // Message("Correct.");

        //use q = Qubit[8];
        //ApplyXorInPlace(4, q);
        //ModularMultiplyByConstant(11, 7, q);
        //DumpMachine();
        //ResetAll(q);


        //DumpMachine();
        //ResetAll(q+[t]);
        return [];
    }
}
