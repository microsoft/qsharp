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
        let nTotal = 10000;
        let thresholdInconcl = 0.8;
        let thresholdConcl = 0.1;
        mutable isCorrect = true;

        // counts total inconclusive answers
        mutable nInconc = 0;

        // counts total conclusive |0⟩ state identifications
        mutable nConclOne = 0;

        // counts total conclusive |+> state identifications
        mutable nConclPlus = 0;

        use qs = Qubit[1];
        for i in 1..nTotal {

            // get a random integer to define the state of the qubits
            let state = DrawRandomInt(0, 1);

            // do state prep: convert |0⟩ to outcome with return equal to state
            SetQubitZeroOrPlus(qs[0], state);

            // get the solution's answer and analyze it
            let ans = Kata.IsQubitZeroPlusOrInconclusive(qs[0]);

            // keep track of the number of inconclusive answers given
            if ans == -1 {
                set nInconc += 1;
            }

            if (ans == 0 and state == 0) {
                set nConclOne += 1;
            }

            if (ans == 1 and state == 1) {
                set nConclPlus += 1;
            }

            ResetAll(qs);
        }

        if IntAsDouble(nInconc) > thresholdInconcl * IntAsDouble(nTotal) {
            Message($"{nInconc} test runs out of {nTotal} returned inconclusive which does not meet the required threshold of at most {thresholdInconcl * 100.0}%.");
            set isCorrect = false;
        }

        if IntAsDouble(nConclOne) < thresholdConcl * IntAsDouble(nTotal) {
            Message($"{nConclOne} test runs out of {nTotal} returned conclusive |0⟩ which does not meet the required threshold of at least {thresholdConcl * 100.0}%.");
            set isCorrect = false;
        }

        if IntAsDouble(nConclPlus) < thresholdConcl * IntAsDouble(nTotal) {
            Message($"{nConclPlus} test runs out of {nTotal} returned conclusive |+⟩ which does not meet the required threshold of at least {thresholdConcl * 100.0}%.");
            set isCorrect = false;
        }

        if (isCorrect) {
            Message("Correct!");
            return true;
        } else {
            Message("Incorrect");
            return false;
        }
    }

}
