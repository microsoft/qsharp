namespace Kata.Verification {
    import Std.Convert.*;
    import KatasUtils.*;

    operation ZeroAndBitstringSuperposition_Reference(qs : Qubit[], bits : Bool[]) : Unit is Adj + Ctl {
        H(qs[0]);

        for i in 1..Length(qs) - 1 {
            if bits[i] {
                CNOT(qs[0], qs[i]);
            }
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let bitstrings = [
            [true],
            [true, false],
            [true, true],
            [true, false, false],
            [true, false, true],
            [true, true, false],
            [true, true, true]
        ];

        for bits in bitstrings {
            Message($"Testing for bits = {bits}...");
            if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
                Kata.ZeroAndBitstringSuperposition(_, bits),
                ZeroAndBitstringSuperposition_Reference(_, bits),
                Length(bits)
            ) {
                return false;
            }
        }

        true
    }
}
