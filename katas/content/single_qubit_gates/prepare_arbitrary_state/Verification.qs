namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;

    operation PrepareArbitraryState (alpha : Double, beta : Double, theta : Double, q : Qubit) : Unit is Adj+Ctl {
        let phi = ArcTan2(beta, alpha);
        Ry(2.0 * phi, q);
        R1(theta, q);
    }

    operation CheckSolution() : Bool {
         for i in 0 .. 10 {
            let i = IntAsDouble(i);
            let alpha = Cos(i);
            let beta = Sin(i);
            let theta = Sin(i);
            let op = (qubit) => Kata.PrepareArbitraryState(alpha, beta, theta, qubit);
            let reference = (qubit) => PrepareArbitraryState(alpha, beta, theta, qubit);
            let isCorrect = VerifySingleQubitOperation(op, reference);
            if not isCorrect {
                Message("Incorrect.");
                Message("The solution was incorrect for at least one test case.");
                use target = Qubit[1];
                let opOnRegister = register => op(register[0]);
                let referenceOnRegister = register => reference(register[0]);
                ShowQuantumStateComparison(target, opOnRegister, referenceOnRegister);
                return false;
            }
        }

        Message("Correct!");
        Message("The solution was correct for all test cases.");
        true
    }
}
