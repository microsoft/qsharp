// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Arithmetic {
    open Microsoft.Quantum.Canon;
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Diagnostics;

    /// # Summary
    /// Applies `X` operations to qubits in a little-endian register for
    /// corresponding 1 bits in an integer.
     operation ApplyXorInPlace(value : Int, target : Qubit[]) : Unit is Adj+Ctl {
        body(...) {
            Fact(value >= 0, "value must be non-negative");
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
    /// Automatically chooses between addition with
    /// carry and without, depending on the register size of `ys`,
    /// which holds the result after operation is complete.
    operation AddI (xs: Qubit[], ys: Qubit[]) : Unit is Adj + Ctl {
        if xs::Length == ys::Length {
            RippleCarryAdderNoCarryTTK(xs, ys);
        }
        elif ys::Length > xs::Length {
            use qs = Qubit[ys::Length - xs::Length - 1];
            RippleCarryAdderTTK(xs + qs, Most(ys), Tail(ys));
        }
        else {
            fail "xs must not contain more qubits than ys!";
        }
    }

    /// # Summary
    /// Reversible, in-place ripple-carry addition of two integers without carry out.
    ///
    /// # Description
    /// Given two $n$-bit integers encoded in LittleEndian registers `xs` and `ys`,
    /// the operation computes the sum of the two integers modulo $2^n$,
    /// where $n$ is the bit size of the inputs `xs` and `ys`. It does not compute
    /// the carry out bit.
    ///
    /// # Input
    /// ## xs
    /// LittleEndian qubit register encoding the first integer summand.
    /// ## ys
    /// LittleEndian qubit register encoding the second integer summand, is
    /// modified to hold the $n$ least significant bits of the sum.
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
        Fact(xs::Length == ys::Length,
            "Input registers must have the same number of qubits." );

        if (xs::Length > 1) {
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
    /// Given two $n$-bit integers encoded in LittleEndian registers `xs` and `ys`,
    /// and a qubit carry, the operation computes the sum of the two integers
    /// where the $n$ least significant bits of the result are held in `ys` and
    /// the carry out bit is xored to the qubit `carry`.
    ///
    /// # Input
    /// ## xs
    /// LittleEndian qubit register encoding the first integer summand.
    /// ## ys
    /// LittleEndian qubit register encoding the second integer summand, is
    /// modified to hold the $n$ least significant bits of the sum.
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
    /// This operation has the same functionality as RippleCarryAdderD and,
    /// RippleCarryAdderCDKM but does not use any ancilla qubits.
    operation RippleCarryAdderTTK(xs : Qubit[], ys : Qubit[], carry : Qubit)
    : Unit is Adj + Ctl {
        Fact(xs::Length == ys::Length,
            "Input registers must have the same number of qubits." );

        if (xs::Length > 1) {
            CNOT(xs[xs::Length-1], carry);
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
    /// the inner operation to construct the full adder.
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
        Fact(xs::Length == ys::Length,
            "Input registers must have the same number of qubits." );
        for i in 1..xs::Length-1 {
            CNOT(xs[i], ys[i]);
        }
        for i in xs::Length-2..-1..1 {
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
            Fact(xs::Length == ys::Length,
                "Input registers must have the same number of qubits." );

            for idx in 0..xs::Length - 2 {
                CCNOT (xs[idx], ys[idx], xs[idx + 1]);
            }
            for idx in xs::Length-1..-1..1 {
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
            Fact(xs::Length == ys::Length,
                "Input registers must have the same number of qubits." );

            let nQubits = xs::Length; 
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

}
