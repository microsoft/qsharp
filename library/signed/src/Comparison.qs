// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Std.Arrays.Tail, Std.Arrays.Zipped, Std.Arrays.Most, Std.Arrays.Rest;
import Std.Diagnostics.Fact;
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


export
    CompareGTSI,
    CompareGTI,
    GreaterThan;