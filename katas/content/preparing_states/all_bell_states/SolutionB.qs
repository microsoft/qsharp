namespace Kata {
    import Std.Convert.*;

    operation AllBellStates(qs : Qubit[], index : Int) : Unit is Adj + Ctl {
        let bitmask = IntAsBoolArray(index, 2);

        if bitmask[0] {
            X(qs[0]);
        }

        if bitmask[1] {
            X(qs[1]);
        }

        H(qs[0]);
        CNOT(qs[0], qs[1]);
    }
}
