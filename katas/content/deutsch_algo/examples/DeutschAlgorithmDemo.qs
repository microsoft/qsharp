namespace Kata {
    import Std.Diagnostics.*;
    import Std.Math.*;

    @EntryPoint()
    operation DeutschAlgorithmDemo() : Unit {
        for (oracle, name) in [
            (PhaseOracle_Zero, "f(x) = 0"),
            (PhaseOracle_One, "f(x) = 1"),
            (PhaseOracle_X, "f(x) = x"),
            (PhaseOracle_OneMinusX, "f(x) = 1-x")
        ] {
            let isConstant = DeutschAlgorithm(oracle);
            Message($"{name} identified as {isConstant ? "constant" | "variable"}");
        }
    }

    operation PhaseOracle_Zero(x : Qubit) : Unit {}

    operation PhaseOracle_One(x : Qubit) : Unit {
        R(PauliI, 2.0 * PI(), x);
    }

    operation PhaseOracle_X(x : Qubit) : Unit {
        Z(x);
    }

    operation PhaseOracle_OneMinusX(x : Qubit) : Unit {
        Z(x);
        R(PauliI, 2.0 * PI(), x);
    }

    operation DeutschAlgorithm(oracle : Qubit => Unit) : Bool {
        use x = Qubit();
        H(x);
        oracle(x);
        H(x);
        return MResetZ(x) == Zero;
    }
}
