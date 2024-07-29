namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Random;

    @EntryPoint()
    operation CheckSolution() : Bool {
        use q = Qubit();
        for _ in 1 .. 4 {
            // repeat 4 times since we are testing a measurement and wrong basis still might get
            // the correct answer, reduces probability of false positives
            if (Kata.AliceQuantum(false, q) != false) {
                Message("Measuring |0⟩ in the Z basis returned incorrect value; expected false");
                Reset(q);
                return false;
            }

            // apply the Pauli X gate
            X(q);
            if (Kata.AliceQuantum(false, q) != true) {
                Message("Measuring |1⟩ in the Z basis returned incorrect value; expected true");
                Reset(q);
                return false;
            }

            // apply the Hadamard gate
            H(q);
            if (Kata.AliceQuantum(true, q) != false) {
                Message("Measuring |+⟩ in the X basis returned incorrect value; expected false");
                Reset(q);
                return false;
            }

            // apply the Pauli X and then the Hadamard gate
            X(q);
            H(q);
            if (Kata.AliceQuantum(true, q) != true) {
                Message("Measuring |-⟩ in the X basis returned incorrect value; expected true");
                Reset(q);
                return false;
            }
            Reset(q);
        }
        Message("Correct!");
        true
    }
}
