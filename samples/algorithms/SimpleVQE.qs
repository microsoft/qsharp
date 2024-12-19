/// # Sample
/// A part of ansatz circuit used in water molecule simulation
/// This sample is suitable for adaptive profile.
///
/// # Description
/// This is an example of the quantum part of a Variational Quantum Eigensolver (VQE) method.
/// This example shows one variant of ansatz circuit used in a water molecule simulation
/// in the referenced paper. To keep this sample suitable for adaptive profile,
/// random angles are used and optimization part is omitted.
///
/// # Reference
/// Ground-state energy estimation of the water molecule on a trapped ion quantum
/// computer by Yunseong Nam et al., 2019. https://arxiv.org/abs/1902.10171

import Std.Diagnostics.Fact;

/// # Summary
/// Bosonic exitation circuit
operation BosonicExitationTerm(
    theta: Double,
    moX: Qubit,
    moY: Qubit): Unit {

    Adjoint S(moX);
    Rxx(theta, moX, moY);
    S(moX);
    Adjoint S(moY);
    Rxx(-theta, moX, moY);
    S(moY);
}

/// # Summary
/// Non-bosonic exitation circuit
operation NonBosonicExitataionTerm(
    theta: Double,
    moXsoX : Qubit,
    moXsoY : Qubit,
    moYsoX : Qubit,
    moYsoY : Qubit) : Unit {

    Adjoint S(moXsoX);
    within {
        CNOT(moXsoX, moYsoY);
        CNOT(moXsoX, moYsoX);
        CNOT(moXsoX, moXsoY);
        H(moXsoX);
    } apply {
        Rz(theta, moXsoX);
        CNOT(moXsoY, moXsoX);
        Rz(theta, moXsoX);
        CNOT(moYsoY, moXsoX);
        Rz(-theta, moXsoX);
        CNOT(moXsoY, moXsoX);
        Rz(-theta, moXsoX);
        Adjoint S(moYsoX);
        CNOT(moYsoX, moXsoX);
        Rx(theta, moXsoX);
        CNOT(moXsoY, moXsoX);
        Rx(theta, moXsoX);
        CNOT(moYsoY, moXsoX);
        Rz(-theta, moXsoX);
        CNOT(moXsoY, moXsoX);
        Rz(-theta, moXsoX);
    }
    S(moYsoX);
}

/// # Summary
/// Prepare state containing both bosonic and non-bosinic interactions
/// And measure the result in PauliZ basis.
operation PrepareAndMeasureAnsatzInZ(
    thetasMo: Double[],
    thetasSo: Double[]) : Result[] {

    Fact(Length(thetasMo) == 4, "Length of thetasMo should be 4.");
    Fact(Length(thetasSo) == 4, "Length of thetasSo should be 4.");

    use mo = Qubit[4];

    BosonicExitationTerm(thetasMo[0], mo[0], mo[1]);
    BosonicExitationTerm(thetasMo[1], mo[2], mo[3]);
    BosonicExitationTerm(thetasMo[2], mo[0], mo[3]);
    BosonicExitationTerm(thetasMo[3], mo[1], mo[2]);

    use so = Qubit[4];
    CNOT(mo[0], so[0]);
    CNOT(mo[1], so[1]);
    CNOT(mo[2], so[2]);
    CNOT(mo[3], so[3]);

    NonBosonicExitataionTerm(thetasSo[0], mo[0], so[0], mo[1], so[1]);
    NonBosonicExitataionTerm(thetasSo[1], mo[2], so[2], mo[3], so[3]);
    NonBosonicExitataionTerm(thetasSo[2], mo[1], so[1], mo[2], so[2]);
    NonBosonicExitataionTerm(thetasSo[3], mo[0], so[0], mo[3], so[3]);

    MResetEachZ(mo+so)
}

/// # Summary
/// Count number of zeroes when performing measurements
/// on a state determined by some predefined angles.
operation FindZeroFrequencies(shots: Int): Int[] {
    let thetasMo = [0.5, 0.5, 0.5, 0.5];
    let thetasSo = [0.3, 0.3, 0.3, 0.3];
    mutable c0 = 0;
    mutable c1 = 0;
    mutable c2 = 0;
    mutable c3 = 0;
    mutable c4 = 0;
    mutable c5 = 0;
    mutable c6 = 0;
    mutable c7 = 0;
    for _ in 1..shots {
        let results = PrepareAndMeasureAnsatzInZ(thetasMo, thetasSo);
        set c0 = c0 + if results[0] == Zero {1} else {0};
        set c1 = c1 + if results[1] == Zero {1} else {0};
        set c2 = c2 + if results[2] == Zero {1} else {0};
        set c3 = c3 + if results[3] == Zero {1} else {0};
        set c4 = c4 + if results[4] == Zero {1} else {0};
        set c5 = c4 + if results[5] == Zero {1} else {0};
        set c6 = c4 + if results[6] == Zero {1} else {0};
        set c7 = c4 + if results[7] == Zero {1} else {0};
    }
    [c0, c1, c2, c3, c4, c5, c6, c7]
}

/// # Summary
/// Prepare one ansatz state and perform measurements multiple times
/// In actual VQE state parameters will be varied based on measurement
/// results.
operation Main() : Int[] {
    FindZeroFrequencies(1000)
}
