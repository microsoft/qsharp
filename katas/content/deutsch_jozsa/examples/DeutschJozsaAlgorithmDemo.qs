namespace Kata {
    import Std.Diagnostics.*;
    import Std.Math.*;

    @EntryPoint()
    operation DeutschJozsaAlgorithmDemo() : Unit {
        for (N, oracle, name) in [
            (2, qs => (), "f(x) = 0"),
            (2, qs => R(PauliI, 2.0 * PI(), qs[0]), "f(x) = 1"),
            (3, qs => Z(qs[Length(qs) - 1]), "f(x) = x mod 2"),
            (3, qs => Z(qs[0]), "f(x) = most significant bit of x"),
            (3, ApplyToEach(Z, _), "f(x) = parity of x")
        ] {
            let isConstant = DeutschJozsaAlgorithm(N, oracle);
            Message($"{name} identified as {isConstant ? "constant" | "balanced"}");
        }
    }

    operation DeutschJozsaAlgorithm(N : Int, oracle : Qubit[] => Unit) : Bool {
        mutable isConstant = true;
        use x = Qubit[N];
        ApplyToEach(H, x);
        oracle(x);
        ApplyToEach(H, x);
        for xi in x {
            if MResetZ(xi) == One {
                set isConstant = false;
            }
        }
        return isConstant;
    }
}
