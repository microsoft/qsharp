namespace Kata.Verification {
    import Std.Arrays.*;
    import Std.Convert.*;
    import Std.Logical.*;
    import Std.Random.*;

    function WinCondition_Reference(rst : Bool[], abc : Bool[]) : Bool {
        return (rst[0] or rst[1] or rst[2]) == (abc[0] != abc[1] != abc[2]);
    }

    // All possible starting bits (r, s and t) that the referee can give
    // to Alice, Bob and Charlie.
    function RefereeBits() : Bool[][] {
        return [
            [false, false, false],
            [true, true, false],
            [false, true, true],
            [true, false, true]
        ];
    }

    operation PlayClassicalGHZ_Reference(strategies : (Bool => Bool)[], inputs : Bool[]) : Bool[] {
        let r = inputs[0];
        let s = inputs[1];
        let t = inputs[2];
        let a = strategies[0](r);
        let b = strategies[1](s);
        let c = strategies[2](t);
        return [a, b, c];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let inputs = RefereeBits();
        let strategies = [Kata.AliceClassical, Kata.BobClassical, Kata.CharlieClassical];

        let iterations = 1000;
        mutable wins = 0;
        for _ in 0..iterations - 1 {
            for bits in inputs {
                let abc = PlayClassicalGHZ_Reference(strategies, bits);
                if WinCondition_Reference(bits, abc) {
                    set wins = wins + 1;
                }
            }
        }
        // The solution is correct if the players win 75% (3/4) of the time.
        if wins < iterations * Length(inputs) * 3 / 4 {
            Message($"Alice, Bob, and Charlie's classical strategy gets {wins} wins out of {iterations * Length(inputs)} possible inputs, which is not optimal");
            return false;
        }
        Message("Correct!");
        true
    }
}
