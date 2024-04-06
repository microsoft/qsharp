namespace Kata {
    operation AmplitudeChange (alpha : Double, q : Qubit) : Unit is Adj + Ctl {
        Ry(2.0 * alpha, q);
    }
}