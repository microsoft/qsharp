namespace Kata {
    import Std.Math.*;

    operation PrepareArbitraryState(alpha : Double, beta : Double, theta : Double, q : Qubit) : Unit is Adj + Ctl {
        let phi = ArcTan2(beta, alpha);
        Ry(2.0 * phi, q);
        R1(theta, q);
    }
}
