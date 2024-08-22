// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Std.Arrays.Tail, Std.Arrays.Zipped, Std.Arrays.Most, Std.Arrays.Rest;
import Utils.ApplyCCNOTChain;

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
    within {
        CNOT(Tail(xs), tmp);
        CNOT(Tail(ys), tmp);
    } apply {
        X(tmp);
        Controlled CompareGTI([tmp], (xs, ys, result));
        X(tmp);
        CCNOT(tmp, Tail(ys), result);    
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

/// # Summary
/// Applies a greater-than comparison between two integers encoded into
/// qubit registers, flipping a target qubit based on the result of the
/// comparison.
///
/// # Description
/// Carries out a strictly greater than comparison of two integers $x$ and $y$, encoded
/// in qubit registers xs and ys. If $x > y$, then the result qubit will be flipped,
/// otherwise the result qubit will retain its state.
///
/// # Input
/// ## xs
/// LittleEndian qubit register encoding the first integer $x$.
/// ## ys
/// LittleEndian qubit register encoding the second integer $y$.
/// ## result
/// Single qubit that will be flipped if $x > y$.
///
/// # References
/// - Steven A. Cuccaro, Thomas G. Draper, Samuel A. Kutin, David
///   Petrie Moulton: "A new quantum ripple-carry addition circuit", 2004.
///   https://arxiv.org/abs/quant-ph/0410184v1
///
/// - Thomas Haener, Martin Roetteler, Krysta M. Svore: "Factoring using 2n+2 qubits
///     with Toffoli based modular multiplication", 2016
///     https://arxiv.org/abs/1611.07995
///
/// # Remarks
/// Uses the trick that $x - y = (x'+y)'$, where ' denotes the one's complement.
operation GreaterThan(xs : Qubit[], ys : Qubit[], result : Qubit) : Unit is Adj + Ctl {
    body (...) {
        (Controlled GreaterThan)([], (xs, ys, result));
    }
    controlled (controls, ...) {
        let nQubits = Length(xs);

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

export
    CompareGTSI,
    CompareGTI,
    GreaterThan;