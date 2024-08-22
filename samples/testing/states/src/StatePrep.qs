/// # Summary
/// Prepares the state 1/2 (|00⟩ + i|01⟩ - |10⟩ - i|11⟩) = |-⟩ ⊗ |i⟩.
operation PrepareStateWithComplexPhases(qs : Qubit[]) : Unit {
    H(qs[0]);
    Z(qs[0]);
    H(qs[1]);
    S(qs[1]);
}

/// # Summary
/// Prepares the state 1/2 (-|00⟩ - i|01⟩ + |10⟩ + i|11⟩) = -|-⟩ ⊗ |i⟩.
operation PrepareStateWithGlobalPhase(qs : Qubit[]) : Unit {
    H(qs[0]);
    Z(qs[0]);
    X(qs[0]);
    H(qs[1]);
    S(qs[1]);
}
