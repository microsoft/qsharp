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
            let i = IntAsDouble(i);

            if not VerifyDoubleDoubleSingleQubitOperation(Cos(i), Sin(i),  Kata.PrepareRotatedState, PrepareRotatedState) {
                return false;
            }
        }
        return true;
    }
}
