namespace Kata.Verification {

    // All possible starting bits (r, s and t) that the referee can give
    // to Alice, Bob and Charlie.
    function RefereeBits () : Bool[][] {
        return [[false, false, false],
                [true, true, false],
                [false, true, true],
                [true, false, true]];
    }

    operation AliceClassical (x : Bool) : Bool {
        return true;
    }

    operation BobClassical (y : Bool) : Bool {
        return true;
    }

    operation CharlieClassical (z : Bool) : Bool {
        return true;
    }

    operation PlayClassicalGHZ_Reference (strategies : (Bool => Bool)[], inputs : Bool[]) : Bool[] {
        mutable results : (Bool)[] = [];
        for i in 0 .. Length(strategies) - 1{
            set results += [strategies[i](inputs[i])];
        }
        return results;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        // To test the interaction, run it on several deterministic strategies (not necessarily good ones)
        let inputs = RefereeBits();
        let strategies = [AliceClassical, BobClassical, CharlieClassical];
        for rst in inputs {
            let actual = Kata.PlayClassicalGHZ(strategies, rst);
            let expected = PlayClassicalGHZ_Reference(strategies, rst);
            if actual != expected {
                Message($"Expected {expected}, got {actual} for {rst}");
                return false;
            }
        }
        true
    }
}
