namespace Kata {
    import Std.Math.*;
    operation IsQubitNotInABC(q : Qubit) : Int {
        let alpha = ArcCos(Sqrt(2.0 / 3.0));

        use a = Qubit();
        Z(q);
        CNOT(a, q);
        Controlled H([q], a);
        S(a);
        X(q);

        ApplyControlledOnInt(0, Ry, [a], (-2.0 * alpha, q));
        CNOT(a, q);
        Controlled H([q], a);
        CNOT(a, q);

        let res0 = MResetZ(a);
        let res1 = M(q);

        if (res0 == Zero and res1 == Zero) {
            return 0;
        } elif (res0 == One and res1 == Zero) {
            return 1;
        } elif (res0 == Zero and res1 == One) {
            return 2;
        } else {
            // this should never occur
            return 3;
        }
    }
}
