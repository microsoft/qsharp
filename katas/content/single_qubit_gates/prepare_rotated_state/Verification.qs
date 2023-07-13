namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Canon;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Katas;

    operation PrepareRotatedState (alpha : Double, beta : Double, q : Qubit) : Unit is Adj+Ctl {
        let phi = ArcTan2(beta, alpha);
        Rx(2.0 * phi, q);
    }



    operation CheckSolution() : Bool {
        for i in 0 .. 10 {
            let op = register => Kata.PrepareRotatedState(Cos(IntAsDouble(i)), Sin(IntAsDouble(i)), register);
            let reference = register => PrepareRotatedState(Cos(IntAsDouble(i)), Sin(IntAsDouble(i)), register);

            if not VerifySingleQubitOperation(op, reference) {
                return false;
            }
        }
        return true;
    }
}
