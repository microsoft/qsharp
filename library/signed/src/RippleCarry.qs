// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Std.Diagnostics.Fact;
import Operations.Sum;
import Std.Arrays.Most, Std.Arrays.Rest, Std.Arrays.Zipped;

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
operation ApplyOuterCDKMAdder(xs : Qubit[], ys : Qubit[], ancilla : Qubit) : Unit is Adj + Ctl {
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
operation CarryOutCoreCDKM(
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
operation ApplyInnerTTKAdder(xs : Qubit[], ys : Qubit[], carry : Qubit) : Unit is Adj + Ctl {
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
operation ApplyOuterTTKAdder(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl {
    let nQubits = Length(xs);

    Fact(
        nQubits == Length(ys),
        "Input registers must have the same number of qubits."
    );

    ApplyToEachCA(CNOT, Zipped(Rest(xs), Rest(ys)));
    Adjoint ApplyCNOTChain(Rest(xs));
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





export RippleCarryAdderNoCarryTTK, ApplyInnerTTKAdderWithoutCarry, RippleCarryAdderTTK, RippleCarryAdderCDKM, RippleCarryAdderD, Carry;