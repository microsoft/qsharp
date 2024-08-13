import Microsoft.Quantum.Math.Min;
import Microsoft.Quantum.Diagnostics.CheckAllZero;
// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import Std.Arrays.Tail, Std.Arrays.Most;
import Std.Core.Length;
import Std.Diagnostics.Fact;

/// # Summary
/// Wrapper for signed integer comparison: `result = xs > ys`.
///
/// # Input
/// ## xs
/// First $n$-bit number
/// ## ys
/// Second $n$-bit number
/// ## result
/// Will be flipped if $xs > ys$
operation CompareGTSI(xs : Qubit[], ys : Qubit[], result : Qubit) : Unit is Adj + Ctl {
    use tmp = Qubit();
    CNOT(Tail(xs), tmp);
    CNOT(Tail(ys), tmp);
    X(tmp);
    Controlled CompareGTI([tmp], (xs, ys, result));
    X(tmp);
    CCNOT(tmp, Tail(ys), result);
    CNOT(Tail(xs), tmp);
    CNOT(Tail(ys), tmp);
}

operation DivideI(xs : Qubit[], ys : Qubit[], result : Qubit[]) : Unit is Adj + Ctl {
    body (...) {
        Controlled DivideI([], (xs, ys, result));
    }
    controlled (controls, ...) {
        let n = Length(result);

        Fact(n == Length(ys), "Integer division requires
                           equally-sized registers ys and result.");
        Fact(n == Length(xs), "Integer division
                            requires an n-bit dividend registers.");

        let xpadded = xs + result;

        for i in (n - 1)..(-1)..0 {
            let xtrunc = xpadded[i..i + n-1];
            Controlled CompareGTI(controls, (ys, xtrunc, result[i]));
            // if ys > xtrunc, we don't subtract:
            (Controlled X)(controls, result[i]);
            (Controlled Adjoint AddI)([result[i]], (ys, xtrunc));
        }
    }
}   


/// # Summary
/// Inverts a given integer modulo 2's complement.
///
/// # Input
/// ## xs
/// n-bit signed integer (SignedLittleEndian), will be inverted modulo
/// 2's complement.
operation Invert2sSI(xs : Qubit[]) : Unit is Adj + Ctl {
    body (...) {
        Controlled Invert2sSI([], xs);
    }
    controlled (controls, ...) {
        ApplyToEachCA((Controlled X)(controls, _), xs);

        use aux = Qubit[Length(xs)];
        within {
            Controlled X(controls, aux[0]);
        } apply {
            AddI(aux, xs);
        }
    }
}


    /// # Summary
/// Computes the reciprocal 1/x for an unsigned integer x
/// using integer division. The result, interpreted as an integer,
/// will be `floor(2^(2*n-1) / x)`.
///
/// # Input
/// ## xs
/// n-bit unsigned integer
/// ## result
/// 2n-bit output, must be in $\ket{0}$ initially.
///
/// # Remarks
/// For the input x=0, the output will be all-ones.
operation ComputeReciprocalI(xs : Qubit[], result : Qubit[]) : Unit is Adj + Ctl {
    body (...) {
        Controlled ComputeReciprocalI([], (xs, result));
    }
    controlled (controls, ...) {
        let n = Length(xs);
        Fact(Length(result) == 2 * n, "Result register must contain 2n qubits.");
        use lhs = Qubit[2 * n];
        use padding = Qubit[n];
        let paddedxs = xs + padding;
        X(Tail(lhs)); // initialize left-hand side to 2^{2n-1}
        // ... and divide:
        (Controlled DivideI)(controls, (lhs, paddedxs, result));
        // uncompute lhs
        for i in 0..2 * n - 1 {
            (Controlled AddI)([result[i]], (paddedxs[0..2 * n-1-i], lhs[i..2 * n-1]));
        }
        X(Tail(lhs));
    }
}



    /// # Summary
/// Automatically chooses between addition with
/// carry and without, depending on the register size of `ys`.
///
/// # Input
/// ## xs
/// $n$-bit addend.
/// ## ys
/// Addend with at least $n$ qubits. Will hold the result.
operation AddI(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
    if Length(xs) == Length(ys) {
        RippleCarryAdderNoCarryTTK(xs, ys);
    } elif Length(ys) > Length(xs) {
        use qs = Qubit[Length(ys) - Length(xs) - 1];
        RippleCarryAdderTTK(xs + qs, Most(ys), Tail(ys));
    } else {
        fail "xs must not contain more qubits than ys";
    }
}

    /// # Summary
/// Wrapper for integer comparison: `result = x > y`.
///
/// # Input
/// ## xs
/// First $n$-bit number
/// ## ys
/// Second $n$-bit number
/// ## result
/// Will be flipped if $x > y$
operation CompareGTI(xs : Qubit[], ys : Qubit[], result : Qubit) : Unit is Adj + Ctl {
    GreaterThan(xs, ys, result);
}



// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.


open Microsoft.Quantum.Intrinsic;
open Microsoft.Quantum.Canon;
open Microsoft.Quantum.Diagnostics;
open Microsoft.Quantum.Arrays;

/// # Summary
/// Implements a reversible carry gate. Given a carry-in bit encoded in
/// qubit `carryIn` and two summand bits encoded in `summand1` and `summand2`,
/// computes the bitwise xor of `carryIn`, `summand1` and `summand2` in the
/// qubit `summand2` and the carry-out is xored to the qubit `carryOut`.
///
/// # Input
/// ## carryIn
/// Carry-in qubit.
/// ## summand1
/// First summand qubit.
/// ## summand2
/// Second summand qubit, is replaced with the lower bit of the sum of
/// `summand1` and `summand2`.
/// ## carryOut
/// Carry-out qubit, will be xored with the higher bit of the sum.
operation Carry(carryIn : Qubit, summand1 : Qubit, summand2 : Qubit, carryOut : Qubit) : Unit is Adj + Ctl {
    CCNOT(summand1, summand2, carryOut);
    CNOT(summand1, summand2);
    CCNOT(carryIn, summand2, carryOut);
}

    /// # Summary
/// Implements a reversible sum gate. Given a carry-in bit encoded in
/// qubit `carryIn` and two summand bits encoded in `summand1` and `summand2`,
/// computes the bitwise xor of `carryIn`, `summand1` and `summand2` in the qubit
/// `summand2`.
///
/// # Input
/// ## carryIn
/// Carry-in qubit.
/// ## summand1
/// First summand qubit.
/// ## summand2
/// Second summand qubit, is replaced with the lower bit of the sum of
/// `summand1` and `summand2`.
///
/// # Remarks
/// In contrast to the `Carry` operation, this does not compute the carry-out bit.
operation Sum(carryIn : Qubit, summand1 : Qubit, summand2 : Qubit) : Unit is Adj + Ctl {
    CNOT(summand1, summand2);
    CNOT(carryIn, summand2);
}

    /// # Summary
/// Reversible, in-place ripple-carry addition of two integers.
///
/// # Description
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
/// - Thomas G. Draper: "Addition on a Quantum Computer", 2000.
///   https://arxiv.org/abs/quant-ph/0008033
///
/// # Remarks
/// The specified controlled operation makes use of symmetry and mutual
/// cancellation of operations to improve on the default implementation
/// that adds a control to every operation.
operation RippleCarryAdderD(xs : Qubit[], ys : Qubit[], carry : Qubit) : Unit is Adj + Ctl {
    body (...) {
        Controlled RippleCarryAdderD([], (xs, ys, carry));
    }
    controlled (controls, ...) {
        let nQubits = Length(xs);

        Fact(
            nQubits == Length(ys),
            "Input registers must have the same number of qubits."
        );

        use auxRegister = Qubit[nQubits];
        for idx in 0..(nQubits-2) {
            Carry(auxRegister[idx], xs[idx], ys[idx], auxRegister[idx + 1]);           // (1)
        }
        Controlled Carry(controls, (auxRegister[nQubits-1], xs[nQubits-1], ys[nQubits-1], carry));
        Controlled CNOT(controls, (xs[nQubits-1], ys[nQubits-1]));
        Controlled Sum(controls, (auxRegister[nQubits-1], xs[nQubits-1], ys[nQubits-1]));
        for idx in (nQubits-2)..(-1)..0 {
            Adjoint Carry(auxRegister[idx], xs[idx], ys[idx], auxRegister[idx + 1]); // cancels with (1)
            Controlled Sum(controls, (auxRegister[idx], xs[idx], ys[idx]));
        }
    }
}

    /// # Summary
/// Reversible, in-place ripple-carry operation that is used in the
/// integer addition operation RippleCarryAdderCDKM below.
/// Given two qubit registers `xs` and `ys` of the same length, the operation
/// applies a ripple carry sequence of CNOT and CCNOT gates with qubits
/// in `xs` and `ys` as the controls and qubits in `xs` as the targets.
///
/// # Input
/// ## xs
/// First qubit register, containing controls and targets.
/// ## ys
/// Second qubit register, contributing to the controls.
/// ## ancilla
/// The ancilla qubit used in RippleCarryAdderCDKM passed to this operation.
///
/// # References
/// - Steven A. Cuccaro, Thomas G. Draper, Samuel A. Kutin, David
///   Petrie Moulton: "A new quantum ripple-carry addition circuit", 2004.
///   https://arxiv.org/abs/quant-ph/0410184v1
internal operation ApplyOuterCDKMAdder(xs : Qubit[], ys : Qubit[], ancilla : Qubit) : Unit is Adj + Ctl {
    let nQubits = Length(xs);

    Fact(
        nQubits == Length(ys),
        "Input registers must have the same number of qubits."
    );

    Fact(
        nQubits >= 3,
        "Need at least 3 qubits per register."
    );

    CNOT(xs[2], xs[1]);
    CCNOT(ancilla, ys[1], xs[1]);
    for idx in 2..nQubits - 2 {
        CNOT(xs[idx + 1], xs[idx]);
        CCNOT(xs[idx-1], ys[idx], xs[idx]);
    }
}

    /// # Summary
/// The core operation in the RippleCarryAdderCDKM, used with the above
/// ApplyOuterCDKMAdder operation, i.e. conjugated with this operation to obtain
/// the inner operation of the RippleCarryAdderCDKM. This operation computes
/// the carry out qubit and applies a sequence of NOT gates on part of the input `ys`.
///
/// # Input
/// ## xs
/// First qubit register.
/// ## ys
/// Second qubit register.
/// ## ancilla
/// The ancilla qubit used in RippleCarryAdderCDKM passed to this operation.
/// ## carry
/// Carry out qubit in the RippleCarryAdderCDKM operation.
///
/// # References
/// - Steven A. Cuccaro, Thomas G. Draper, Samuel A. Kutin, David
///   Petrie Moulton: "A new quantum ripple-carry addition circuit", 2004.
///   https://arxiv.org/abs/quant-ph/0410184v1
internal operation CarryOutCoreCDKM(
    xs : Qubit[],
    ys : Qubit[],
    ancilla : Qubit,
    carry : Qubit
) : Unit is Adj + Ctl {
    let nQubits = Length(xs);

    Fact(
        nQubits == Length(ys),
        "Input registers must have the same number of qubits."
    );

    CNOT(xs[nQubits - 1], carry);
    CCNOT(xs[nQubits - 2], ys[nQubits - 1], carry);
    ApplyToEachCA(X, Most(Rest(ys)));   // X on ys[1..(nQubits-2)]
    CNOT(ancilla, ys[1]);
    ApplyToEachCA(CNOT, Zipped(Rest(Most(xs)), Rest(Rest(ys))));
}

    /// # Summary
/// Reversible, in-place ripple-carry addition of two integers.
///
/// # Description
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
/// modified to hold the n least significant bits of the sum.
/// ## carry
/// Carry qubit, is xored with the most significant bit of the sum.
///
/// # References
/// - Steven A. Cuccaro, Thomas G. Draper, Samuel A. Kutin, David
///   Petrie Moulton: "A new quantum ripple-carry addition circuit", 2004.
///   https://arxiv.org/abs/quant-ph/0410184v1
///
/// # Remarks
/// This operation has the same functionality as RippleCarryAdderD, but
/// only uses one auxiliary qubit instead of $n$.
operation RippleCarryAdderCDKM(xs : Qubit[], ys : Qubit[], carry : Qubit) : Unit is Adj + Ctl {
    let nQubits = Length(xs);

    Fact(
        nQubits == Length(ys),
        "Input registers must have the same number of qubits."
    );

    use auxiliary = Qubit();
    within {
        ApplyToEachCA(CNOT, Zipped(Rest(xs), Rest(ys)));
        CNOT(xs[1], auxiliary);
        CCNOT(xs[0], ys[0], auxiliary);
        ApplyOuterCDKMAdder(xs, ys, auxiliary);
    } apply {
        CarryOutCoreCDKM(xs, ys, auxiliary, carry);
    }
    ApplyToEachCA(X, Most(Rest(ys)));
    CNOT(xs[0], ys[0]);
}

// /// # Summary
// /// Implements the inner addition function for the operation
// /// RippleCarryAdderTTK. This is the inner operation that is conjugated
// /// with the outer operation to construct the full adder.
// ///
// /// # Input
// /// ## xs
// /// LittleEndian qubit register encoding the first integer summand
// /// input to RippleCarryAdderTTK.
// /// ## ys
// /// LittleEndian qubit register encoding the second integer summand
// /// input to RippleCarryAdderTTK.
// /// ## carry
// /// Carry qubit, is xored with the most significant bit of the sum.
// ///
// /// # References
// /// - Yasuhiro Takahashi, Seiichiro Tani, Noboru Kunihiro: "Quantum
// ///   Addition Circuits and Unbounded Fan-Out", Quantum Information and
// ///   Computation, Vol. 10, 2010.
// ///   https://arxiv.org/abs/0910.2530
// ///
// /// # Remarks
// /// The specified controlled operation makes use of symmetry and mutual
// /// cancellation of operations to improve on the default implementation
// /// that adds a control to every operation.
internal operation ApplyInnerTTKAdder(xs : Qubit[], ys : Qubit[], carry : Qubit) : Unit is Adj + Ctl {
    body (...) {
        (Controlled ApplyInnerTTKAdder)([], (xs, ys, carry));
    }
    controlled (controls, ...) {
        let nQubits = Length(xs);

        for idx in 0..nQubits - 2 {
            CCNOT(xs[idx], ys[idx], xs[idx + 1]);
        }
        (Controlled CCNOT)(controls, (xs[nQubits-1], ys[nQubits-1], carry));
        for idx in nQubits - 1..-1..1 {
            Controlled CNOT(controls, (xs[idx], ys[idx]));
            CCNOT(xs[idx-1], ys[idx-1], xs[idx]);
        }
    }
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
internal operation ApplyOuterTTKAdder(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
    let nQubits = Length(xs);

    Fact(
        nQubits == Length(ys),
        "Input registers must have the same number of qubits."
    );

    ApplyToEachCA(CNOT, Zipped(Rest(xs), Rest(ys)));
    Adjoint ApplyCNOTChain(Rest(xs));
}

// /// # Summary
// /// Reversible, in-place ripple-carry addition of two integers.
// /// Given two $n$-bit integers encoded in LittleEndian registers `xs` and `ys`,
// /// and a qubit carry, the operation computes the sum of the two integers
// /// where the $n$ least significant bits of the result are held in `ys` and
// /// the carry out bit is xored to the qubit `carry`.
// ///
// /// # Input
// /// ## xs
// /// LittleEndian qubit register encoding the first integer summand.
// /// ## ys
// /// LittleEndian qubit register encoding the second integer summand, is
// /// modified to hold the $n$ least significant bits of the sum.
// /// ## carry
// /// Carry qubit, is xored with the most significant bit of the sum.
// ///
// /// # References
// /// - Yasuhiro Takahashi, Seiichiro Tani, Noboru Kunihiro: "Quantum
// ///   Addition Circuits and Unbounded Fan-Out", Quantum Information and
// ///   Computation, Vol. 10, 2010.
// ///   https://arxiv.org/abs/0910.2530
// ///
// /// # Remarks
// /// This operation has the same functionality as RippleCarryAdderD and,
// /// RippleCarryAdderCDKM but does not use any ancilla qubits.
operation RippleCarryAdderTTK(xs : Qubit[], ys : Qubit[], carry : Qubit) : Unit is Adj + Ctl {
    let nQubits = Length(xs);


    if (nQubits > 1) {
        CNOT(xs[nQubits-1], carry);
        within {
            ApplyOuterTTKAdder(xs, ys);
        } apply {
            ApplyInnerTTKAdder(xs, ys, carry);
        }
    } else {
        CCNOT(xs[0], ys[0], carry);
    }
    CNOT(xs[0], ys[0]);
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
operation ApplyInnerTTKAdderWithoutCarry(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
    body (...) {
        (Controlled ApplyInnerTTKAdderWithoutCarry)([], (xs, ys));
    }
    controlled (controls, ...) {
        let nQubits = Length(xs);

        Fact(
            nQubits == Length(ys),
            "Input registers must have the same number of qubits."
        );

        for idx in 0..nQubits - 2 {
            CCNOT(xs[idx], ys[idx], xs[idx + 1]);
        }
        for idx in nQubits - 1..-1..1 {
            Controlled CNOT(controls, (xs[idx], ys[idx]));
            CCNOT(xs[idx - 1], ys[idx - 1], xs[idx]);
        }
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
operation RippleCarryAdderNoCarryTTK(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
    let nQubits = Length(xs);

    Fact(
        nQubits == Length(ys),
        "Input registers must have the same number of qubits."
    );

    if (nQubits > 1) {
        within {
            ApplyOuterTTKAdder(xs, ys);
        } apply {
            ApplyInnerTTKAdderWithoutCarry(xs, ys);
        }
    }
    CNOT(xs[0], ys[0]);
}

// /// # Summary
// /// Applies a greater-than comparison between two integers encoded into
// /// qubit registers, flipping a target qubit based on the result of the
// /// comparison.
// ///
// /// # Description
// /// Carries out a strictly greater than comparison of two integers $x$ and $y$, encoded
// /// in qubit registers xs and ys. If $x > y$, then the result qubit will be flipped,
// /// otherwise the result qubit will retain its state.
// ///
// /// # Input
// /// ## xs
// /// LittleEndian qubit register encoding the first integer $x$.
// /// ## ys
// /// LittleEndian qubit register encoding the second integer $y$.
// /// ## result
// /// Single qubit that will be flipped if $x > y$.
// ///
// /// # References
// /// - Steven A. Cuccaro, Thomas G. Draper, Samuel A. Kutin, David
// ///   Petrie Moulton: "A new quantum ripple-carry addition circuit", 2004.
// ///   https://arxiv.org/abs/quant-ph/0410184v1
// ///
// /// - Thomas Haener, Martin Roetteler, Krysta M. Svore: "Factoring using 2n+2 qubits
// ///     with Toffoli based modular multiplication", 2016
// ///     https://arxiv.org/abs/1611.07995
// ///
// /// # Remarks
// /// Uses the trick that $x - y = (x'+y)'$, where ' denotes the one's complement.
operation GreaterThan(xs : Qubit[], ys : Qubit[], result : Qubit) : Unit is Adj + Ctl {
    body (...) {
        (Controlled GreaterThan)([], (xs, ys, result));
    }
    controlled (controls, ...) {
        let nQubits = Length(xs);

        Fact(
            nQubits == Length(ys),
            "Input registers must have the same number of qubits."
        );

        if (nQubits == 1) {
            X(ys[0]);
            (Controlled CCNOT)(controls, (xs[0], ys[0], result));
            X(ys[0]);
        } else {
            within {
                ApplyToEachCA(X, ys);
                ApplyToEachCA(CNOT, Zipped(Rest(xs), Rest(ys)));
            } apply {
                within {
                    (Adjoint ApplyCNOTChain)(Rest(xs));
                    ApplyCCNOTChain(Most(ys), xs);
                } apply {
                    (Controlled CCNOT)(controls, (xs[nQubits-1], ys[nQubits-1], result));
                }
                (Controlled CNOT)(controls, (xs[nQubits-1], result));
            }
        }
    }
}




    /// # Summary
/// Implements a cascade of CCNOT gates controlled on corresponding bits of two
/// qubit registers, acting on the next qubit of one of the registers.
/// Starting from the qubits at position 0 in both registers as controls, CCNOT is
/// applied to the qubit at position 1 of the target register, then controlled by
/// the qubits at position 1 acting on the qubit at position 2 in the target register,
/// etc., ending with an action on the target qubit in position `Length(nQubits)-1`.
///
/// # Input
/// ## register
/// Qubit register, only used for controls.
/// ## targets
/// Qubit register, used for controls and as target.
///
/// # Remarks
/// The target qubit register must have one qubit more than the other register.
operation ApplyCCNOTChain(register : Qubit[], targets : Qubit[]) : Unit is Adj + Ctl {
    let nQubits = Length(targets);

    Fact(
        nQubits == Length(register) + 1,
        "Target register must have one more qubit."
    );

    ApplyToEachCA(CCNOT, Zipped3(register, Most(targets), Rest(targets)));
}

function Zipped3<'T1, 'T2, 'T3>(first : 'T1[], second : 'T2[], third : 'T3[]) : ('T1, 'T2, 'T3)[] {
    let nElements = Min([Length(first), Length(second), Length(third)]);
    if nElements == 0 {
        return [];
    }
    mutable output = [(first[0], second[0], third[0]), size = nElements];
    for idxElement in 1..nElements - 1 {
        set output w/= idxElement <- (first[idxElement], second[idxElement], third[idxElement]);
    }
    return output;
}
operation MultiplySI(xs : Qubit[], ys : Qubit[], result : Qubit[]) : Unit {
    body (...) {
        Controlled MultiplySI([], (xs, ys, result));
    }
    controlled (controls, ...) {
        use signx = Qubit();
        use signy = Qubit();

        within {
            CNOT(Tail(xs), signx);
            CNOT(Tail(ys), signy);
            Controlled Invert2sSI([signx], xs);
            Controlled Invert2sSI([signy], ys);
        } apply {
            Controlled MultiplyI(controls, (xs, ys, result));
            within {
                CNOT(signx, signy);
            } apply {
                // No controls required since `result` will still be zero
                // if we did not perform the multiplication above.
                Controlled Invert2sSI([signy], result);
            }
        }
    }
    adjoint auto;
    controlled adjoint auto;
}
operation MultiplyI(xs : Qubit[], ys : Qubit[], result : Qubit[]) : Unit is Adj + Ctl {
    body (...) {
        let na = Length(xs);
        let nb = Length(ys);

        for (idx, actl) in Enumerated(xs) {
            Controlled AddI([actl], (ys, (result[idx..idx + nb])));
        }
    }
    controlled (controls, ...) {
        let na = Length(xs);
        let nb = Length(ys);

        // Perform various optimizations based on number of controls
        let numControls = Length(controls);
        if numControls == 0 {
            MultiplyI(xs, ys, result);
        } elif numControls == 1 {
            use aux = Qubit();
            for (idx, actl) in Enumerated(xs) {
                within {
                    AND(controls[0], actl, aux);
                } apply {
                    Controlled AddI([aux], (ys, (result[idx..idx + nb])));
                }
            }
        } else {
            use helper = Qubit[numControls];
            within {
                AndLadder(controls, Most(helper));
            } apply {
                for (idx, actl) in Enumerated(xs) {
                    within {
                        AND(Tail(Most(helper)), actl, Tail(helper));
                    } apply {
                        Controlled AddI([Tail(helper)], (ys, (result[idx..idx + nb])));
                    }
                }
            }
        }
    }
}

operation SquareSI(xs : Qubit[], result : Qubit[]) : Unit is Adj + Ctl {
    body (...) {
        Controlled SquareSI([], (xs, result));
    }
    controlled (controls, ...) {
        let n = Length(xs);
        use signx = Qubit();
        use signy = Qubit();

        within {
            CNOT(Tail(xs), signx);
            Controlled Invert2sSI([signx], xs);
        } apply {
            Controlled SquareI(controls, (xs, result));
        }
    }
}

operation SquareI(xs : Qubit[], result : Qubit[]) : Unit {
    body (...) {
        Controlled SquareI([], (xs, result));
    }
    controlled (controls, ...) {
        let n = Length(xs);


        let numControls = Length(controls);
        if numControls == 0 {
            use aux = Qubit();
            for (idx, ctl) in Enumerated(xs) {
                within {
                    CNOT(ctl, aux);
                } apply {
                    Controlled AddI([aux], (xs, (result[idx..idx + n])));
                }
            }
        } elif numControls == 1 {
            use aux = Qubit();
            for (idx, ctl) in Enumerated(xs) {
                within {
                    AND(controls[0], ctl, aux);
                } apply {
                    Controlled AddI([aux], (xs, (result[idx..idx + n])));
                }
            }
        } else {
            use helper = Qubit[numControls];
            within {
                AndLadder(controls, Most(helper));
            } apply {
                for (idx, ctl) in Enumerated(xs) {
                    within {
                        AND(Tail(Most(helper)), ctl, Tail(helper));
                    } apply {
                        Controlled AddI([Tail(helper)], (xs, (result[idx..idx + n])));
                    }
                }
            }
        }
    }
    adjoint auto;
    controlled adjoint auto;
}



operation AndLadder(controls : Qubit[], targets : Qubit[]) : Unit is Adj {
    let controls1 = [Head(controls)] + Most(targets);
    let controls2 = Rest(controls);
    for (a, b, c) in Zipped3(controls1, controls2, targets) {
        AND(a, b, c);
    }
}


