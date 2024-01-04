// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Unstable.TableLookup {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Measurement;
    open Microsoft.Quantum.ResourceEstimation;
    open Microsoft.Quantum.Unstable.Arithmetic;

    @Config(Full)
    internal operation BasicSelect(
        data : Bool[][],
        address : Qubit[],
        target : Qubit[]
    ) : Unit is Adj + Ctl {
        body (...) {
            let (N, n) = DimensionsForSelect(data, address);

            if N == 1 { // base case
                WriteMemoryContents(Head(data), target);
            } else {
                let (most, tail) = MostAndTail(address[...n - 1]);
                let parts = Partitioned([2^(n - 1)], data);

                within {
                    X(tail);
                } apply {
                    Controlled BasicSelect([tail], (parts[0], most, target));
                }

                Controlled BasicSelect([tail], (parts[1], most, target));
            }
        }
        controlled (ctls, ...) {
            Fact(Length(ctls) == 1, "BasicSelect can only be controlled by one qubit.");
            let ctl = Head(ctls);
            let (N, n) = DimensionsForSelect(data, address);

            if BeginEstimateCaching("Microsoft.Quantum.Unstable.TableLookup.SinglyControlledSelect", N) {
                if N == 1 { // base case
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

                    Controlled BasicSelect([helper], (parts[0], most, target));

                    CNOT(ctl, helper);

                    Controlled BasicSelect([helper], (parts[1], most, target));

                    Adjoint ApplyAndAssuming0Target(ctl, tail, helper);
                }

                EndEstimateCaching();
            }
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
    /// [1] [arXiv:1805.03662](https://arxiv.org/abs/1805.03662)
    ///     "Encoding Electronic Spectra in Quantum Circuits with Linear T
    ///      Complexity"
    /// [2] [arXiv:1905.07682](https://arxiv.org/abs/1905.07682)
    ///     "Windowed arithmetic"
    /// [3] [arXiv:2211.01133](https://arxiv.org/abs/2211.01133)
    ///     "Space-time optimized table lookup"
    @Config(Full)
    operation Select(
        data : Bool[][],
        address : Qubit[],
        target : Qubit[]
    ) : Unit is Adj + Ctl {
        body (...) {
            BasicSelect(data, address, target);
        }
        adjoint (...) {
            Unlookup(Select, data, address, target);
        }

        controlled (ctls, ...) {
            Controlled ApplyAsSinglyControlled(ctls, (BasicSelect, (data, address, target)));
        }

        controlled adjoint (ctls, ...) {
            Controlled Select(ctls, (data, address, target));
        }
    }

    internal function DimensionsForSelect(
        data : Bool[][],
        address : Qubit[]
    ) : (Int, Int) {
        let N = Length(data);
        Fact(N > 0, "data cannot be empty");

        let n = Ceiling(Lg(IntAsDouble(N)));
        Fact(
            Length(address) >= n,
            $"address register is too small, requires at least {n} qubits");

        return (N, n);
    }

    internal operation WriteMemoryContents(
        value : Bool[],
        target : Qubit[]
    ) : Unit is Adj + Ctl {
        Fact(
            Length(value) == Length(target),
            "number of data bits must equal number of target qubits");

        ApplyPauliFromBitString(PauliX, true, value, target);
    }

    /// # References
    /// - [arXiv:1905.07682](https://arxiv.org/abs/1905.07682)
    ///   "Windowed arithmetic"
    @Config(Full)
    internal operation Unlookup(
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
            $"l = {l} must be smaller than {numAddressBits}");

        let res = Mapped(r -> r == One, ForEach(MResetX, target));

        let dataFixup = Chunks(2^l, Padded(-2^numAddressBits, false,
                               Mapped(MustBeFixed(res, _), data)));

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

    // Checks whether specific bitstring `data` must be fixed for a given
    // measurement result `result`.
    //
    // Returns true if the number of indices for which both result and data are
    // `true` is odd.
    internal function MustBeFixed(result : Bool[], data : Bool[]) : Bool {
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
    internal operation EncodeUnary(
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
}
