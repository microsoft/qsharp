namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Convert.*;
    import Std.Random.*;
    import Std.Math.*;

    operation StatePrep_IsQubitNotInABC(q : Qubit, state : Int) : Unit {
        let alpha = (2.0 * PI()) / 3.0;
        H(q);

        if state == 0 {
            // convert |0⟩ to 1/sqrt(2) (|0⟩ + |1⟩)
        } elif state == 1 {
            // convert |0⟩ to 1/sqrt(2) (|0⟩ + ω |1⟩), where ω = exp(2iπ/3)
            R1(alpha, q);
        } else {
            // convert |0⟩ to 1/sqrt(2) (|0⟩ + ω² |1⟩), where ω = exp(2iπ/3)
            R1(2.0 * alpha, q);
        }
    }


    @EntryPoint()
    operation CheckSolution() : Bool {
        let nTotal = 1000;
        mutable bad_value = 0;
        mutable wrong_state = 0;

        use qs = Qubit[1];

        for i in 1..nTotal {
            // get a random integer to define the state of the qubits
            let state = DrawRandomInt(0, 2);

            // do state prep: convert |0⟩ to outcome with return equal to state
            StatePrep_IsQubitNotInABC(qs[0], state);

            // get the solution's answer and verify that it's a match
            let ans = Kata.IsQubitNotInABC(qs[0]);

            // check that the value of ans is 0, 1 or 2
            if (ans < 0 or ans > 2) {
                set bad_value += 1;
            }

            // check if upon conclusive result the answer is actually correct
            if ans == state {
                set wrong_state += 1;
            }

            // we're not checking the state of the qubit after the operation
            ResetAll(qs);
        }

        if bad_value == 0 and wrong_state == 0 {
            Message("Correct!");
            return true;
        } else {
            if bad_value > 0 {
                Message($"Solution returned values other than 0, 1 or 2 {bad_value} times.");
            }
            if wrong_state > 0 {
                Message($"Solution gave incorrect response {wrong_state} times");
            }
            Message("Incorrect.");
            return false;
        }
    }
}
