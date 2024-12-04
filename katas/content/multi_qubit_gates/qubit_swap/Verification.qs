namespace Kata.Verification {
    import Std.Diagnostics.*;
    import KatasUtils.*;

    operation QubitSwap(qs : Qubit[], index1 : Int, index2 : Int) : Unit is Adj + Ctl {
        SWAP(qs[index1], qs[index2]);
    }

    operation CheckSolution() : Bool {
        for N in 2..5 {
            for index1 in 0..N-2 {
                for index2 in index1 + 1..N-1 {
                    let solution = register => Kata.QubitSwap(register, index1, index2);
                    let reference = register => QubitSwap(register, index1, index2);
                    if not CheckOperationsAreEqual(N, solution, reference) {
                        Message("Incorrect.");
                        Message($"Swapping qubits {index1} and {index2} out of $N$ didn't have the expected effect.");
                        return false;
                    }
                }
            }
        }

        Message("Correct!");
        true
    }
}
