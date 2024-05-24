namespace Kata {
    open Microsoft.Quantum.Arithmetic;
    open Microsoft.Quantum.Characterization;
    open Microsoft.Quantum.Oracles;

    operation OraclePowerWrapper (U : (Qubit => Unit is Adj + Ctl), power : Int, target : Qubit[]) : Unit is Adj + Ctl {
        for _ in 1 .. power {
            U(target[0]);
        }
    }

    operation DistinguishYfromXZWithPhases (unitary : (Qubit => Unit is Adj + Ctl)) : Int {
        // Run phase estimation on the unitary and the +1 eigenstate of the Y gate |0⟩ + i|1⟩

        // Construct a phase estimation oracle from the unitary
        let oracle = DiscreteOracle(OraclePowerWrapper(unitary, _, _));

        // Allocate qubits to hold the eigenstate of U and the phase in a big endian register 
        mutable phaseInt = 0;
        use (eigenstate, phaseRegister) = (Qubit[1], Qubit[2]);
        let phaseRegisterBE = BigEndian(phaseRegister);
        // Prepare the eigenstate of U
        H(eigenstate[0]); 
        S(eigenstate[0]);
        // Call library
        QuantumPhaseEstimation(oracle, eigenstate, phaseRegisterBE);
        // Read out the phase
        set phaseInt = MeasureInteger(BigEndianAsLittleEndian(phaseRegisterBE));

        ResetAll(eigenstate);
        ResetAll(phaseRegister);

        // Convert the measured phase into return value
        return phaseInt;
    }
}
