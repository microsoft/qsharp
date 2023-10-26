namespace Kata.Verification {

    // Two qubit parity Measurement
    operation StatePrep_ParityMeasurement(qs : Qubit[], state : Int, alpha : Double) : Unit is Adj {
        // prep cos(alpha) * |0..0⟩ + sin(alpha) * |1..1⟩
        Ry(2.0 * alpha, qs[0]);
        for i in 1 .. Length(qs) - 1 {
            CNOT(qs[0], qs[i]);
        }

        if state == 1 {
            // flip the state of the first half of the qubits
            for i in 0 .. Length(qs) / 2 - 1 {
                X(qs[i]);
            }
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        return DistinguishStates_MultiQubit(
            2,
            2,
            StatePrep_ParityMeasurement,
            Kata.ParityMeasurement,
            true,
            ["α|00⟩ + β|11⟩", "α|01⟩ + β|10⟩"]);
    }
}
