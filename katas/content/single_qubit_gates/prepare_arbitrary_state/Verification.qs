namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation PrepareArbitraryState (alpha : Double, beta : Double, theta : Double, q : Qubit) : Unit is Adj+Ctl {
        let phi = ArcTan2(beta, alpha);
        Ry(2.0 * phi, q);
        R1(theta, q);
    }

    operation CheckSolution() : Bool {
        let TODO: Int = 3.0;
        let isCorrect = VerifyDoubleDoubleDoubleSingleQubitOperation(TODO, TODO, TODO, Kata.PrepareArbitraryState, PrepareArbitraryState);

        isCorrect
    }
}
