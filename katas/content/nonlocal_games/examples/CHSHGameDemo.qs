namespace Quantum.Kata.CHSHGame {
    import Std.Random.*;
    import Std.Math.*;
    import Std.Convert.*;

    function WinCondition(x : Bool, y : Bool, a : Bool, b : Bool) : Bool {
        return (x and y) == (a != b);
    }

    function AliceClassical(x : Bool) : Bool {
        return false;
    }

    function BobClassical(y : Bool) : Bool {
        return false;
    }

    operation AliceQuantum(bit : Bool, qubit : Qubit) : Bool {
        if bit {
            return MResetX(qubit) == One;
        }
        return MResetZ(qubit) == One;
    }

    operation BobQuantum(bit : Bool, qubit : Qubit) : Bool {
        let angle = 2.0 * PI() / 8.0;
        Ry(not bit ? -angle | angle, qubit);
        return M(qubit) == One;
    }

    @EntryPoint()
    operation CHSH_GameDemo() : Unit {
        use (aliceQubit, bobQubit) = (Qubit(), Qubit());
        mutable classicalWins = 0;
        mutable quantumWins = 0;
        let iterations = 1000;
        for _ in 1..iterations {
            H(aliceQubit);
            CNOT(aliceQubit, bobQubit);
            let (x, y) = (DrawRandomBool(0.5), DrawRandomBool(0.5));
            if WinCondition(x, y, AliceClassical(x), BobClassical(y)) {
                set classicalWins += 1;
            }
            if WinCondition(x, y, AliceQuantum(x, aliceQubit), BobQuantum(y, bobQubit)) {
                set quantumWins += 1;
            }
            ResetAll([aliceQubit, bobQubit]);
        }
        Message($"Percentage of classical wins is {100.0 * IntAsDouble(classicalWins) / IntAsDouble(iterations)}%");
        Message($"Percentage of quantum wins is {100.0 * IntAsDouble(quantumWins) / IntAsDouble(iterations)}%");
    }
}
