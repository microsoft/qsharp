/// # Summary
/// Prepares the qubits in the given basis state, measures them, and returns the measurement result.
operation MeasureBasisState(bits : Bool[]) : Result[] {
    use qs = Qubit[Length(bits)];
    ApplyPauliFromBitString(PauliX, true, bits, qs);
    return MResetEachZ(qs);
}