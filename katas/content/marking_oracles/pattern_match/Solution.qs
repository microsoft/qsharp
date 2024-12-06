namespace Kata {
    import Std.Arrays.*;

    operation Oracle_PatternMatching(x : Qubit[], y : Qubit, a : Int[], r : Bool[]) : Unit is Adj + Ctl {
        let ctrl = Subarray(a, x);
        ApplyControlledOnBitString(r, X, ctrl, y);
    }
}
