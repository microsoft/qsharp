// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.
namespace Microsoft.Quantum.Applications.Cryptography {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.ResourceEstimation;
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Unstable.Arithmetic;
    open Microsoft.Quantum.Unstable.TableLookup;

    @EntryPoint()
    operation EstimateEkeraHastad() : Unit {
        // 1024-bit
        EkeraHastad(1024, 135066410865995223349603216278805969938881475605667027524485143851526510604859533833940287150571909441798207282164471551373680419703964191743046496589274256239341020864383202110372958725762358509643110564073501508187510676594629205563685529475213500852879416377328533906109750544334999811150056977236890927563L, 7L);

        // 2048-bit
        // EkeraHastad(2048, 25195908475657893494027183240048398571429282126204032027777137836043662020707595556264018525880784406918290641249515082189298559149176184502808489120072844992687392807287776735971418347270261896375014971824691165077613379859095700097330459748808428401797429100642458691817195118746121515172654632282216869987549182422433637259085141865462043576798423387184774447920739934236584823824281198163815010674810451660377306056201619676256133844143603833904414952634432190114657544454178424020924616515723350778707749817125772467962926386356373289912154831438167899885040445364023527381951378636564391212010397122822120720357L, 7L);
    }

    operation EkeraHastad(numBits : Int, N : BigInt, g : BigInt) : Unit {
        let x = ExpModL(g, ((N - 1L) / 2L), N);
        let xinv = InverseModL(x, N);

        let m = numBits / 2;
        use c1 = Qubit[2 * m];
        use c2 = Qubit[m];
        use target = Qubit[numBits];

        let ne = 3 * m;
        let cpad = Ceiling(2.0 * Lg(IntAsDouble(numBits)) + Lg(IntAsDouble(ne)) + 10.0);

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

    internal function ExponentWindowLength_() : Int { 5 }

    internal function MultiplicationWindowLength_() : Int { 5 }

    // ------------------------------- //
    // Modular arithmetic (operations) //
    // ------------------------------- //

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

    // Computes zs *= (base ^ xs) % mod (for a large register xs)
    internal operation MultiplyExpMod(
        base : BigInt,
        mod : BigInt,
        xs : Qubit[],
        zs : Qubit[]
    ) : Unit {
        let expWindows = Chunks(ExponentWindowLength_(), xs);

        for i in IndexRange(expWindows) {
            if BeginEstimateCaching("MultiplyExpMod", Length(expWindows)) {
                let adjustedBase = ExpModL(base, 1L <<< (i * ExponentWindowLength_()), mod);
                MultiplyExpModWindowed(adjustedBase, mod, expWindows[i], zs);

                EndEstimateCaching();
            }
        }
    }

    // Computes zs *= (base ^ xs) % mod (for a small register xs)
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

    // Computes zs += ys * (base ^ xs) % mod (for small registers xs and ys)
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

        for i in IndexRange(factorWindows) {
            if BeginEstimateCaching("AddExpModWindowed", Length(factorWindows)) {
                // compute data for table lookup
                let factorValue = ExpModL(2L, IntAsBigInt(i * MultiplicationWindowLength_()), mod);
                let data = PseudoModularAddExponentialLookupData(factorValue, Length(xs), Length(factorWindows[i]), base, mod, sign, Length(zs));

                use output = Qubit[Length(data[0])];

                within {
                    Select(data, xs + factorWindows[i], output);
                } apply {
                    RippleCarryCGIncByLE(output, zs);
                }

                EndEstimateCaching();
            }
        }
    }

    internal function PseudoModularAddExponentialLookupData(factor : BigInt, expLength : Int, mulLength : Int, base : BigInt, mod : BigInt, sign : Int, numBits : Int) : Bool[][] {
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
}
