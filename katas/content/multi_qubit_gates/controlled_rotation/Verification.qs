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
            if not CheckOperationsAreEqual(2, solution, reference) {
                let precision = 3;
                Message("Incorrect.");
                Message($"The test case for theta={DoubleAsStringWithPrecision(angle, precision)} did not pass");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}