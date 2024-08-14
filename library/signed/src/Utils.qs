// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

// Util functions for interal use by the signed integer library.

import Std.Math.Min;
import Std.Diagnostics.Fact;
import Std.Arrays.Head, Std.Arrays.Tail, Std.Arrays.Most, Std.Arrays.Rest;

operation AndLadder(controls : Qubit[], targets : Qubit[]) : Unit is Adj {
    let controls1 = [Head(controls)] + Most(targets);
    let controls2 = Rest(controls);
    for (a, b, c) in Zipped3(controls1, controls2, targets) {
        AND(a, b, c);
    }
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

