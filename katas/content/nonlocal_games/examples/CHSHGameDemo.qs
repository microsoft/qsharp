// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

//////////////////////////////////////////////////////////////////////
// This file contains reference solutions to all above exercises.
// We recommend that you try to solve the exercises yourself first,
// but feel free to look up the solution if you get stuck.
//////////////////////////////////////////////////////////////////////

namespace Quantum.Kata.CHSHGame {

    open Microsoft.Quantum.Random;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Intrinsic;

    // Win condition for both classical and quantum games
    function WinCondition (x : Bool, y : Bool, a : Bool, b : Bool) : Bool {
        return (x and y) == (a != b);
    }

    //////////////////////////////////////////////////////////////////
    // Classical CHSH
    //////////////////////////////////////////////////////////////////

    // Alice and Bob's classical strategy
    // (Both players should return the same value, regardless of input)
    function AliceClassical (x : Bool) : Bool {
        return false;
    }

    function BobClassical (y : Bool) : Bool {
        return false;
    }

    //////////////////////////////////////////////////////////////////
    // Quantum CHSH
    //////////////////////////////////////////////////////////////////

    // Create entangled pair
    operation CreateEntangledPair (aliceQubit : Qubit, bobQubit : Qubit) : Unit is Adj {
        H(aliceQubit);
        CNOT(aliceQubit, bobQubit);
    }

    // Alice's quantum strategy
    operation AliceQuantum (bit : Bool, qubit : Qubit) : Bool {
        // Measure in sign basis if bit is 1, and
        // measure in computational basis if bit is 0
        if bit {
            let q = MResetX(qubit);
            return (q == One);
        }
        else {
            let q = MResetZ(qubit);
            return (q == One);
        }
    }

    // Bob's quantum strategy
    operation BobQuantum (bit : Bool, qubit : Qubit) : Bool {

        let angle = 2.0 * PI() / 8.0;
        Ry(not bit ? -angle | angle, qubit);
        return M(qubit) == One;
    }

    operation PlayClassicCHSH(x : Bool, y: Bool) : Bool
    {
        let (a, b) = (AliceClassical(x), BobClassical(y));
        Message($"Classical Alice and Bob respond with a={a} and b={b}");
        return WinCondition(x, y, a, b);
    }

    operation PlayQuantumCHSH(x : Bool, y : Bool, aliceQubit: Qubit, bobQubit : Qubit) : Bool
    {
        let (a, b) = (AliceQuantum(x, aliceQubit), BobQuantum(y, bobQubit));
        Message($"Quantum Alice and Bob respond with a={a} and b={b}");
        return WinCondition(x, y, a, b);
    }

    @EntryPoint()
    operation CHSH_GameDemo() : (Bool, Bool) {
// create entaingled pair before the game
        use aliceQubit = Qubit();
        use bobQubit = Qubit();
        CreateEntangledPair(aliceQubit, bobQubit);

        let (x, y) = (DrawRandomBool(0.5), DrawRandomBool(0.5));
        Message($"Referee has bits x={x} and y={y}");
        let isAliceBobWinClassic = PlayClassicCHSH(x, y);
        Message($"Alice and Bob {isAliceBobWinClassic ? "win" | "lose"} classical CHSG game");
        let isAliceBobWinQuantum = PlayQuantumCHSH(x, y, aliceQubit, bobQubit);
        Message($"Alice and Bob {isAliceBobWinQuantum ? "win" | "lose"} quantum CHSG game");

        ResetAll([aliceQubit, bobQubit]);
        return (isAliceBobWinClassic, isAliceBobWinQuantum);
    }
}
