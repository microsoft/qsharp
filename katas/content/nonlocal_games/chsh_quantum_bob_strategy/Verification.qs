namespace Kata.Verification {
    import Std.Math.*;

    operation RotateBobQubit(clockwise : Bool, qubit : Qubit) : Unit {
        if (clockwise) {
            Ry(-PI() / 4.0, qubit);
        } else {
            Ry(PI() / 4.0, qubit);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for _ in 1..4 {
            // repeat 4 times since we are testing a measurement and wrong basis still might get
            // the correct answer, reduces probability of false positives
            use q = Qubit();
            RotateBobQubit(false, q);
            let result = Kata.BobQuantum(false, q);
            Reset(q);
            if (result != false) {
                Message("π/8 from |0⟩ not measured as false");
                return false;
            }

            X(q);
            RotateBobQubit(false, q);
            let result = Kata.BobQuantum(false, q);
            Reset(q);
            if (result != true) {
                Message("π/8 from |1⟩ not measured as true");
                return false;
            }

            RotateBobQubit(true, q);
            let result = Kata.BobQuantum(true, q);
            Reset(q);
            if (result != false) {
                Message("-π/8 from |0⟩ not measured as false");
                return false;
            }

            X(q);
            RotateBobQubit(true, q);
            let result = Kata.BobQuantum(true, q);
            Reset(q);
            if (result != true) {
                Message("-π/8 from |1⟩ not measured as true");
                return false;
            }
        }
        Message("Correct!");
        true
    }
}
