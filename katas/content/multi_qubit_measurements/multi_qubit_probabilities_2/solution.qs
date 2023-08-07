namespace Kata {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;

    operation SetInitialState(qs: Qubit[]): Unit is Adj + Ctl {
        // Next two lines set the second qubit into the desired state
        let second_bit_angle = 2.0 * ArcCos(2.0 / 3.0);
        Ry(second_bit_angle, qs[1]);

        // Next two lines set the first qubit into the desired state
        let first_bit_angle = 2.0 * ArcCos(1.0 / Sqrt(5.0));
        Controlled Ry([qs[1]], (first_bit_angle, qs[0]));
    }

    operation ChangeBasis(qs: Qubit[]): Unit is Adj + Ctl {
        H(qs[0]);
        H(qs[1]);
    }

    @EntryPoint()
    operation CalculateProbabilities(): Unit {
        // This allocates qubits for us to work with
        use qs = Qubit[2];

        SetInitialState(qs);

        // Check that we've prepared the state |𝜓⟩
        Message("The state of the system before the transformation:");
        DumpMachine();

        ChangeBasis(qs);

        Message("Final state of the two-qubit system:");
        DumpMachine();

        // This returns the qubit array into state |00❭
        ResetAll(qs);
    }

}
