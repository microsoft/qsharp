/// # Summary
/// Bell States sample
///
/// # Description
/// This Q# sample shows how to prepare the four different Bell states.
///
/// # Remarks
/// Bell states or EPR pairs are specific quantum states of two qubits
/// that represent the simplest (and maximal) examples of quantum entanglement.
///
/// # References
/// - [Bell state](https://en.wikipedia.org/wiki/Bell_state)
operation Main() : (Result, Result)[] {
    // Prepare and measure each pair. Return an array of these results.
    [
        PrepareAndMeasurePair(PreparePhiPlus, "|Φ+〉"),
        PrepareAndMeasurePair(PreparePhiMinus, "|Φ-〉"),
        PrepareAndMeasurePair(PreparePsiPlus, "|Ψ+〉"),
        PrepareAndMeasurePair(PreparePsiMinus, "|Ψ-〉")
    ]
}

/// # Summary
/// Allocates a pair of qubits, prepares them using `preparation` operation,
/// Then measures and resets them. Returns a pair of results.
operation PrepareAndMeasurePair(
    preparation : Qubit[] => Unit,
    name : String
) : (Result, Result) {
    use pair = Qubit[2]; // Allocate a pair of qubits
    preparation(pair); // Prepare them using the preparation operation
    Message($"Bell state {name}:"); // Show the name of the prepared state
    Std.Diagnostics.DumpMachine(); // Show the prepared state
    (MResetZ(pair[0]), MResetZ(pair[1])) // Measure, reset and return
}

/// # Summary
/// Prepares |Φ+⟩ = (|00⟩+|11⟩)/√2 state assuming `register` is in |00⟩ state.
operation PreparePhiPlus(register : Qubit[]) : Unit {
    H(register[0]);                 // |+0〉
    CNOT(register[0], register[1]); // 1/sqrt(2)(|00〉 + |11〉)
}

/// # Summary
/// Prepares |Φ−⟩ = (|00⟩-|11⟩)/√2 state assuming `register` is in |00⟩ state.
operation PreparePhiMinus(register : Qubit[]) : Unit {
    H(register[0]);                 // |+0〉
    Z(register[0]);                 // |-0〉
    CNOT(register[0], register[1]); // 1/sqrt(2)(|00〉 - |11〉)
}

/// # Summary
/// Prepares |Ψ+⟩ = (|01⟩+|10⟩)/√2 state assuming `register` is in |00⟩ state.
operation PreparePsiPlus(register : Qubit[]) : Unit {
    H(register[0]);                 // |+0〉
    X(register[1]);                 // |+1〉
    CNOT(register[0], register[1]); // 1/sqrt(2)(|01〉 + |10〉)
}

/// # Summary
/// Prepares |Ψ−⟩ = (|01⟩-|10⟩)/√2 state assuming `register` is in |00⟩ state.
operation PreparePsiMinus(register : Qubit[]) : Unit {
    H(register[0]);                 // |+0〉
    Z(register[0]);                 // |-0〉
    X(register[1]);                 // |-1〉
    CNOT(register[0], register[1]); // 1/sqrt(2)(|01〉 - |10〉)
}
