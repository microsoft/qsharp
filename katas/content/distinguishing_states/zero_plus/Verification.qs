namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Convert.*;
    import Std.Random.*;


    operation SetQubitZeroOrPlus(q : Qubit, state : Int) : Unit {
        if state != 0 {
            H(q);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let nTotal = 1000;
        mutable nOk = 0;
        let threshold = 0.8;

        use qs = Qubit[1];
        for i in 1..nTotal {
            // get a random integer to define the state of the qubits
            let state = DrawRandomInt(0, 1);

            // do state prep: convert |0⟩ to outcome with return equal to state
            SetQubitZeroOrPlus(qs[0], state);

            // get the solution's answer and verify that it's a match
            let ans = Kata.IsQubitZeroOrPlus(qs[0]);
            if ans == (state == 0) {
                set nOk += 1;
            }

            // we're not checking the state of the qubit after the operation
            ResetAll(qs);
        }

        if IntAsDouble(nOk) < threshold * IntAsDouble(nTotal) {
            Message($"{nTotal - nOk} test runs out of {nTotal} returned incorrect state, which does not meet the required threshold of at least {threshold * 100.0}%.");
            Message("Incorrect.");
            return false;
        } else {
            Message("Correct!");
            return true;
        }
    }
}
