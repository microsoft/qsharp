namespace Kata.Verification {
    import Std.Convert.*;
    import Std.Random.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        use q = Qubit();
        for _ in 1..4 {
            // repeat 4 times since we are testing a measurement and wrong basis still might get
            // the correct answer, reduces probability of false positives
            let result = Kata.AliceQuantum(false, q);
            Reset(q);
            if (result != false) {
                Message("Measuring |0⟩ in the Z basis returned incorrect value; expected false");
                return false;
            }

            // apply the Pauli X gate
            X(q);
            let result = Kata.AliceQuantum(false, q);
            Reset(q);
            if (result != true) {
                Message("Measuring |1⟩ in the Z basis returned incorrect value; expected true");
                return false;
            }

            // apply the Hadamard gate
            H(q);
            let result = Kata.AliceQuantum(true, q);
            Reset(q);
            if (result != false) {
                Message("Measuring |+⟩ in the X basis returned incorrect value; expected false");
                return false;
            }

            // apply the Pauli X and then the Hadamard gate
            X(q);
            H(q);
            let result = Kata.AliceQuantum(true, q);
            Reset(q);
            if (result != true) {
                Message("Measuring |-⟩ in the X basis returned incorrect value; expected true");
                return false;
            }
        }
        Message("Correct!");
        true
    }
}
