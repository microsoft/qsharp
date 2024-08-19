import Std.Random.DrawRandomDouble;

/// # Summary
/// Apply Pauli noise to a qubit. The probability of applying each Pauli operator is given by the parameters pX, pY, and pZ.
///
/// # Input
/// ## pX
/// The probability of applying the Pauli-X operator.
/// ## pY
/// The probability of applying the Pauli-Y operator.
/// ## pZ
/// The probability of applying the Pauli-Z operator.
/// ## qubit
/// The qubit to which the noise is applied.
///
/// # Remarks
/// The probabilities pX, pY, and pZ must be non-negative and sum to at most 1.0. If the sum is less than 1.0,
/// the identity operator is applied with the remaining probability.
operation ApplyPauliNoise(pX : Double, pY : Double, pZ : Double, qubit : Qubit) : Unit {
    if pX < 0.0 or pY < 0.0 or pZ < 0.0 or pX + pY + pZ > 1.0 {
        fail "Invalid probabilities for Pauli noise.";
    }

    let p = DrawRandomDouble(0.0, 1.0);
    if p < pX {
        X(qubit);
    } elif p < pX + pY {
        Y(qubit);
    } elif p < pX + pY + pZ {
        Z(qubit);
    }
}

/// # Summary
/// Apply bit-flip noise to a qubit. The probability of flipping the qubit is given by the parameter p.
///
/// # Input
/// ## p
/// The probability of flipping the qubit.
/// ## qubit
/// The qubit to which the noise is applied.
///
/// # Remarks
/// The probability p must be non-negative and at most 1.0.
operation ApplyBitFlipNoise(p : Double, qubit : Qubit) : Unit {
    ApplyPauliNoise(p, 0.0, 0.0, qubit);
}

/// # Summary
/// Apply phase-flip noise to a qubit. The probability of flipping the qubit is given by the parameter p.
///
/// # Input
/// ## p
/// The probability of flipping the qubit.
/// ## qubit
/// The qubit to which the noise is applied.
///
/// # Remarks
/// The probability p must be non-negative and at most 1.0.
operation ApplyPhaseFlipNoise(p : Double, qubit : Qubit) : Unit {
    ApplyPauliNoise(0.0, 0.0, p, qubit);
}

/// # Summary
/// Apply depolarizing noise to a qubit. The probability of applying each Pauli operator is p/3.
///
/// # Input
/// ## p
/// The probability of applying depolarizing noise to the qubit, which is split evenly between
/// the Pauli-X, Pauli-Y, and Pauli-Z operators.
/// ## qubit
/// The qubit to which the noise is applied.
///
/// # Remarks
/// The probability p must be non-negative and at most 1.0.
operation ApplyDepolarizingNoise(p : Double, qubit : Qubit) : Unit {
    ApplyPauliNoise(p / 3.0, p / 3.0, p / 3.0, qubit);
}

export ApplyPauliNoise, ApplyBitFlipNoise, ApplyPhaseFlipNoise, ApplyDepolarizingNoise;
