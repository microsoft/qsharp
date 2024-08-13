namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Canon;
    open Microsoft.Quantum.Logical;

    // All possible starting bits (r, s and t) that the referee can give
    // to Alice, Bob and Charlie.
    function RefereeBits () : Bool[][] {
        return [[false, false, false],
                [true, true, false],
                [false, true, true],
                [true, false, true]];
    }

    function WinCondition_Reference (rst : Bool[], abc : Bool[]) : Bool {
        return (rst[0] or rst[1] or rst[2]) == Xor(Xor(abc[0], abc[1]), abc[2]);
    }

    operation AliceQuantum_Reference (bit : Bool, qubit : Qubit) : Bool {
        if bit {
            H(qubit);
        }    
        return M(qubit) == One;
    }

    operation BobQuantum_Reference (bit : Bool, qubit : Qubit) : Bool {
        if bit {
            H(qubit);
        }    
        return M(qubit) == One;
    }

    operation CharlieQuantum_Reference (bit : Bool, qubit : Qubit) : Bool {
        if bit {
            H(qubit);
        }    
        return M(qubit) == One;
    }

    operation CreateEntangledTriple_Reference (qs : Qubit[]) : Unit is Adj {
        X(qs[0]);
        X(qs[1]);

        H(qs[0]);
        H(qs[1]);
        // At this point we have (|000⟩ - |010⟩ - |100⟩ + |110⟩) / 2

        // Flip the sign of the last term
        Controlled Z([qs[0]], qs[1]);

        // Flip the state of the last qubit for the two middle terms
        ApplyControlledOnBitString([false, true], X, [qs[0], qs[1]], qs[2]);
        ApplyControlledOnBitString([true, false], X, [qs[0], qs[1]], qs[2]);
    }

    operation PlayQuantumGHZ_Reference (strategies : ((Bool, Qubit) => Bool)[], inputs : Bool[]) : Bool[] {
        use qs = Qubit[3];
        CreateEntangledTriple_Reference(qs);
        let r = inputs[0];
        let s = inputs[1];
        let t = inputs[2];
        let a = strategies[0](r, qs[0]);
        let b = strategies[1](s, qs[1]);
        let c = strategies[2](t, qs[2]);

        ResetAll(qs);
        return [a, b, c];
    }
    
    @EntryPoint()
    operation CheckSolution() : Bool {
        let inputs = RefereeBits();
        let strategies = [AliceQuantum_Reference, BobQuantum_Reference, CharlieQuantum_Reference];
        for rst in inputs {
            let actualBits = Kata.PlayQuantumGHZ(strategies, rst);
            if Length(actualBits) != 3 {
                Message($"Expected 3 bits from PlayQuantumGHZ, got {Length(actualBits)}");
                return false;
            }
            let expectedBits = PlayQuantumGHZ_Reference(strategies, rst);            
            let actualWin = WinCondition_Reference(rst, actualBits);
            let expectedWin = WinCondition_Reference(rst, expectedBits);
            if actualWin != expectedWin {
                Message($"Expected win={expectedWin} {expectedBits}, got {actualBits} for {rst}");
                return false;
            }
            else {
                Message($"Cool {expectedBits}, got {actualBits} for {rst}");
            }
        }
        true
    }
}
