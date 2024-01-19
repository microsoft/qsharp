// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.
namespace Microsoft.Quantum.Applications.Chemistry {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Unstable.Arithmetic;
    open Microsoft.Quantum.Unstable.TableLookup;

    // ------------------------------------- //
    // State preparation (public operations) //
    // ------------------------------------- //

    operation PrepareSingleQubit(p0 : Double, p1 : Double, target : Qubit) : Unit is Adj + Ctl {
        let oneNorm = p0 + p1;
        let alpha = ArcCos(Sqrt(p0 / oneNorm));

        Ry(2.0 * alpha, target);
    }

    operation PrepareUniformSuperposition(numStates : Int, qs : Qubit[]) : Unit is Adj + Ctl {
        Fact(numStates >= 1, "numStates must be positive");
        Fact(numStates <= 2^Length(qs), $"numStates must be smaller or equal to {2^Length(qs)}");

        let qsAdjusted = qs[...Ceiling(Lg(IntAsDouble(numStates))) - 1];

        let (factor, pow) = DecomposePowerOf2(numStates);

        if factor == 1 {
            ApplyToEachCA(H, qsAdjusted[0..pow - 1]);
        } else {
            use tgt = Qubit();

            let sqrt = Sqrt(IntAsDouble(1 <<< Length(qsAdjusted)) / IntAsDouble(numStates));
            let angle = 2.0 * ArcSin(0.5 * sqrt);

            ApplyToEachCA(H, qsAdjusted);

            ApplyIfGreaterL(Ry(2.0 * angle, _), IntAsBigInt(numStates), qsAdjusted, tgt);

            within {
                ApplyToEachA(H, qsAdjusted[pow...]);
            } apply {
                ReflectAboutInteger(0, qsAdjusted[pow...] + [tgt]);
                Ry(-angle, tgt);
            }

            X(tgt);
        }
    }

    newtype PrepareArbitrarySuperposition = (
        NIndexQubits: Int,
        NGarbageQubits: Int,
        Prepare: (Qubit[], Qubit[], Qubit[]) => Unit is Adj + Ctl,
        PrepareWithSelect: ((Bool[][], Qubit[], Qubit[]) => Unit is Adj + Ctl, Qubit[], Qubit[], Qubit[]) => Unit is Adj + Ctl
    );

    function MakePrepareArbitrarySuperposition(targetError : Double, coefficients : Double[])
    : PrepareArbitrarySuperposition {
        let nBitsPrecision = -Ceiling(Lg(0.5 * targetError)) + 1;
        let positiveCoefficients = Mapped(AbsD, coefficients);
        let (keepCoeff, altIndex) = DiscretizedProbabilityDistribution(nBitsPrecision, positiveCoefficients);
        let nCoeffs = Length(positiveCoefficients);
        let nBitsIndices = Ceiling(Lg(IntAsDouble(nCoeffs)));

        let op = PrepareQuantumROMState(nBitsPrecision, nCoeffs, nBitsIndices, keepCoeff, altIndex, [], Select, _, _, _);
        let opWithSelect = PrepareQuantumROMState(nBitsPrecision, nCoeffs, nBitsIndices, keepCoeff, altIndex, [], _, _, _, _);
        let (nIndexQubits, nGarbageQubits) = ArbitrarySuperpositionRegisterLengths(targetError, nCoeffs);
        return PrepareArbitrarySuperposition(nIndexQubits, nGarbageQubits, op, opWithSelect);
    }

    function MakePrepareArbitrarySuperpositionWithData(targetError : Double, coefficients : Double[], data: Bool[][]) : PrepareArbitrarySuperposition {
        let nBitsPrecision = -Ceiling(Lg(0.5 * targetError)) + 1;
        let positiveCoefficients = Mapped(AbsD, coefficients);
        let (keepCoeff, altIndex) = DiscretizedProbabilityDistribution(nBitsPrecision, positiveCoefficients);
        let nCoeffs = Length(positiveCoefficients);
        let nBitsIndices = Ceiling(Lg(IntAsDouble(nCoeffs)));

        let op = PrepareQuantumROMState(nBitsPrecision, nCoeffs, nBitsIndices, keepCoeff, altIndex, data, Select, _, _, _);
        let opWithSelect = PrepareQuantumROMState(nBitsPrecision, nCoeffs, nBitsIndices, keepCoeff, altIndex, data, _, _, _, _);
        let (nIndexQubits, nGarbageQubits) = ArbitrarySuperpositionRegisterLengths(targetError, nCoeffs);
        return PrepareArbitrarySuperposition(nIndexQubits, nGarbageQubits + Length(data[0]), op, opWithSelect);
    }

    // -------------------------------------- //
    // State preparation (private operations) //
    // -------------------------------------- //

    internal function DecomposePowerOf2(number : Int) : (Int, Int) {
        mutable pow = 0;
        mutable factor = number;

        while factor % 2 == 0 {
            set factor /= 2;
            set pow += 1;
        }

        (factor, pow)
    }

    internal function ArbitrarySuperpositionRegisterLengths(targetError : Double, nCoefficients : Int)
    : (Int, Int) {
        Fact(targetError > 0.0, "targetError must be positive");
        Fact(nCoefficients > 0, "nCoefficients must be positive");

        let nBitsPrecision = -Ceiling(Lg(0.5*targetError)) + 1;
        let nIndexQubits = Ceiling(Lg(IntAsDouble(nCoefficients)));
        let nGarbageQubits = nIndexQubits + 2 * nBitsPrecision + 1;
        (nIndexQubits, nGarbageQubits)
    }

    // Computes discretized probability distribution as described in Section 3
    // and Fig. 13 in [arXiv:1805.03662](https://arxiv.org/pdf/1805.03662.pdf)
    internal function DiscretizedProbabilityDistribution(bitsPrecision: Int, coefficients: Double[])
    : (Int[], Int[]) {
        let oneNorm = PNorm(1.0, coefficients);
        let nCoefficients = Length(coefficients);
        Fact(bitsPrecision <= 31, $"Bits of precision {bitsPrecision} unsupported. Max is 31.");
        Fact(nCoefficients > 1, "Cannot prepare state with less than 2 coefficients.");
        Fact(oneNorm != 0.0, "State must have at least one coefficient > 0");

        let barHeight = 2 ^ bitsPrecision - 1;

        mutable altIndex = SequenceI(0, nCoefficients - 1);
        mutable keepCoeff = Mapped(
            coefficient -> Round((AbsD(coefficient) / oneNorm) * IntAsDouble(nCoefficients) * IntAsDouble(barHeight)),
            coefficients
        );

        // Calculate difference between number of discretized bars vs. maximum
        let bars = Fold((state, value) -> state + value - barHeight, 0, keepCoeff);

        // Uniformly distribute excess bars across coefficients.
        for idx in 0..AbsI(bars) - 1 {
            set keepCoeff w/= idx <- keepCoeff[idx] + (bars > 0 ? -1 | +1);
        }

        mutable barSink = [];
        mutable barSource = [];

        for idxCoeff in IndexRange(keepCoeff) {
            if keepCoeff[idxCoeff] > barHeight {
                set barSource += [idxCoeff];
            } elif keepCoeff[idxCoeff] < barHeight {
                set barSink += [idxCoeff];
            }
        }

        for rep in 0..nCoefficients * 10 {
            if Length(barSink) > 0 and Length(barSource) > 0 {
                let idxSink = Tail(barSink);
                let idxSource = Tail(barSource);
                set barSink = Most(barSink);
                set barSource = Most(barSource);

                set keepCoeff w/= idxSource <- keepCoeff[idxSource] - barHeight + keepCoeff[idxSink];
                set altIndex w/= idxSink <- idxSource;

                if keepCoeff[idxSource] < barHeight {
                    set barSink += [idxSource];
                } elif keepCoeff[idxSource] > barHeight {
                    set barSource += [idxSource];
                }
            } elif Length(barSource) > 0 {
                let idxSource = Tail(barSource);
                set barSource = Most(barSource);
                set keepCoeff w/= idxSource <- barHeight;
            } else {
                return (keepCoeff, altIndex);
            }
        }

        return (keepCoeff, altIndex);
    }

    // Used in QuantumROM implementation.
    internal operation PrepareQuantumROMState(
        nBitsPrecision: Int, nCoeffs: Int, nBitsIndices: Int,
        keepCoeff: Int[], altIndex: Int[], data : Bool[][],
        selectOperation: (Bool[][], Qubit[], Qubit[]) => Unit is Adj + Ctl,
        indexRegister: Qubit[], dataQubits : Qubit[], garbageRegister: Qubit[]
    )
    : Unit is Adj + Ctl {
        let garbageIdx0 = nBitsIndices;
        let garbageIdx1 = garbageIdx0 + nBitsPrecision;
        let garbageIdx2 = garbageIdx1 + nBitsPrecision;
        let garbageIdx3 = garbageIdx2 + 1;

        let altIndexRegister = garbageRegister[0..garbageIdx0 - 1];
        let keepCoeffRegister = garbageRegister[garbageIdx0..garbageIdx1 - 1];
        let uniformKeepCoeffRegister = garbageRegister[garbageIdx1..garbageIdx2 - 1];
        let flagQubit = garbageRegister[garbageIdx3 - 1];
        let dataRegister = dataQubits;
        let altDataRegister = garbageRegister[garbageIdx3...];

        // Create uniform superposition over index and alt coeff register.
        PrepareUniformSuperposition(nCoeffs, indexRegister);
        ApplyToEachCA(H, uniformKeepCoeffRegister);

        // Write bitstrings to altIndex and keepCoeff register.
        let target = keepCoeffRegister + altIndexRegister + dataRegister + altDataRegister;
        let selectData = MappedOverRange(idx ->
                IntAsBoolArray(keepCoeff[idx], Length(keepCoeffRegister)) +
                IntAsBoolArray(altIndex[idx], Length(altIndexRegister)) +
                (IsEmpty(data) ? [] | data[idx] + data[altIndex[idx]]), 0..nCoeffs - 1);
        selectOperation(selectData, indexRegister, target);

        // Perform comparison
        ApplyIfGreaterLE(X, uniformKeepCoeffRegister, keepCoeffRegister, flagQubit);

        let indexRegisterSize = Length(indexRegister);

        // Swap in register based on comparison
        let lhs = indexRegister + dataRegister;
        let rhs = altIndexRegister + altDataRegister;
        for i in IndexRange(lhs) {
            Controlled SWAP([flagQubit], (lhs[i], rhs[i]));
        }
    }
}
