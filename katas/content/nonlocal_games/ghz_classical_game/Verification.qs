namespace Kata.Verification {

    // All possible starting bits (r, s and t) that the referee can give
    // to Alice, Bob and Charlie.
    function RefereeBits () : Bool[][] {
        return [[false, false, false],
                [true, true, false],
                [false, true, true],
                [true, false, true]];
    }

    operation TestStrategy (input : Bool, mode : Int) : Bool {
        return mode == 0 ? false | mode == 1 ? true | mode == 2 ? input | not input;
    }

    operation PlayClassicalGHZ_Reference (strategies : (Bool => Bool)[], inputs : Bool[]) : Bool[] {
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
        for rst in inputs {
            // To test the interaction, run it on deterministic strategies (not necessarily good ones)
            // This logic helps to detect errors in user PlayClassicalGHZ implementation like
            // using the wrong sequence of output bits or not using the strategies at all. 
            for mode_1 in 0 .. 3 {
                for mode_2 in 0 .. 3 {
                    for mode_3 in 0 .. 3 {
                        let strategies = [TestStrategy(_, mode_1), TestStrategy(_, mode_2), TestStrategy(_, mode_3)];
                        let actual = Kata.PlayClassicalGHZ(strategies, rst);
                        let expected = PlayClassicalGHZ_Reference(strategies, rst);
                        if actual != expected {
                            Message($"Expected {expected}, got {actual} for {rst}");
                            return false;
                        }
                    }
                }
            }
        }
        Message("Correct!");
        true
    }
}
