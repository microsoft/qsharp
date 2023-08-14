namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Katas;

    operation ControlledRotation (qs : Qubit[], theta : Double) : Unit is Adj + Ctl {
        let controll = qs[0];
        let target = qs[1];
        Controlled Rx([controll], (theta, target));
    }

    operation CheckSolution() : Bool {
        for i in 0 .. 20 {
            let angle = IntAsDouble(i) / 10.0;
            let solution = register => Kata.ControlledRotation(register, angle);
            let reference = register => ControlledRotation(register, angle);
            if not CheckOperationsEquivalence(solution, reference, 2) {
                Message("Incorrect.");
                Message("At least one test case did not pass");
                return false;
            }
        }

        Message("Correct!");
        Message("All test cases passed.");
        true
    }
}