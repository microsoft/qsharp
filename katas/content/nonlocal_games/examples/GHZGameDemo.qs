namespace Quantum.Kata.GHZGame {
    open Microsoft.Quantum.Random;
    open Microsoft.Quantum.Convert;

    function WinCondition (rst : Bool[], abc : Bool[]) : Bool {
        return (rst[0] or rst[1] or rst[2]) == (abc[0] != abc[1] != abc[2]);
    }

    operation AliceClassical (r : Bool) : Bool {
        return true;
    }

    operation BobClassical (s : Bool) : Bool {
        return true;
    }

    operation CharlieClassical (t : Bool) : Bool {
        return true;
    }

    operation CreateEntangledTriple (qs : Qubit[]) : Unit is Adj {
        X(qs[0]);
        X(qs[1]);
        H(qs[0]);
        H(qs[1]);
        Controlled Z([qs[0]], qs[1]);
        ApplyControlledOnBitString([false, true], X, [qs[0], qs[1]], qs[2]);
        ApplyControlledOnBitString([true, false], X, [qs[0], qs[1]], qs[2]);
    }

    operation AliceQuantum (bit : Bool, qubit : Qubit) : Bool {
        if bit {
            let res = MResetX(qubit);
            return res == One;
        }
        let res = MResetZ(qubit);
        return res == One;
    }

    operation BobQuantum (bit : Bool, qubit : Qubit) : Bool {
        if bit {
            let res = MResetX(qubit);
            return res == One;
        }
        let res = MResetZ(qubit);
        return res == One;
    }

    operation CharlieQuantum (bit : Bool, qubit : Qubit) : Bool {
        if bit {
            let res = MResetX(qubit);
            return res == One;
        }
        let res = MResetZ(qubit);
        return res == One;
    }

    operation getRandomRefereeBits () : Bool[] {
        let bits = [[false, false, false],
                    [true, true, false],
                    [false, true, true],
                    [true, false, true]];
        return bits[DrawRandomInt(0, 3)];
    }

    @EntryPoint()
    operation GHZ_GameDemo () : Unit {
        use (aliceQubit, bobQubit, charlieQubit) = (Qubit(), Qubit(), Qubit());
        mutable classicalWins = 0;
        mutable quantumWins = 0;
        let iterations = 1000;
        for _ in 1 .. iterations {
            CreateEntangledTriple([aliceQubit, bobQubit, charlieQubit]);
            let inputs = getRandomRefereeBits();
            let coutputs = [AliceClassical(inputs[0]), BobClassical(inputs[1]), CharlieClassical(inputs[2])];
            if WinCondition(inputs, coutputs) {
                set classicalWins += 1;
            }
            let qoutputs = [AliceQuantum(inputs[0], aliceQubit), BobQuantum(inputs[1], bobQubit), CharlieQuantum(inputs[2], charlieQubit)];
            if WinCondition(inputs, qoutputs) {
                set quantumWins += 1;
            }
            ResetAll([aliceQubit, bobQubit, charlieQubit]);
        }
        Message($"Percentage of classical wins is {100.0*IntAsDouble(classicalWins)/IntAsDouble(iterations)}%");
        Message($"Percentage of quantum wins is {100.0*IntAsDouble(quantumWins)/IntAsDouble(iterations)}%");
    }
}
