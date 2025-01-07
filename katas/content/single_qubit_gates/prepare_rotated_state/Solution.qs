namespace Kata {
    import Std.Math.*;

    operation PrepareRotatedState(alpha : Double, beta : Double, q : Qubit) : Unit is Adj + Ctl {
        let phi = ArcTan2(beta, alpha);
        Rx(2.0 * phi, q);
    }
}
