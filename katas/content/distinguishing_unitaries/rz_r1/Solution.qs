namespace Kata {
    import Std.Math.*;
    operation DistinguishRzFromR1(unitary : ((Double, Qubit) => Unit is Adj + Ctl)) : Int {
        use qs = Qubit[2];
        H(qs[0]);
        Controlled unitary(qs[0..0], (2.0 * PI(), qs[1]));
        return MResetX(qs[0]) == Zero ? 1 | 0;
    }
}
