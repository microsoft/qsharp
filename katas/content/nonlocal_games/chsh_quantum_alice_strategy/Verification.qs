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
                Message("|0⟩ not measured as false");
                Reset(q);
                return false;
            }

            // apply the Pauli X gate
            X(q);
            if (Kata.AliceQuantum(false, q) != true) {
                Message("|1⟩ not measured as true");
                Reset(q);
                return false;
            }

            // apply the Hadamard gate
            H(q);
            if (Kata.AliceQuantum(true, q) != false) {
                Message("|+⟩ not measured as false");
                Reset(q);
                return false;
            }

            // apply the Pauli X and then the Hadamard gate
            X(q);
            H(q);
            if (Kata.AliceQuantum(true, q) != true) {
                Message("|-⟩ not measured as true");
                Reset(q);
                return false;
            }
            Reset(q);
        }
        Message("Correct!");
        true
    }
}
