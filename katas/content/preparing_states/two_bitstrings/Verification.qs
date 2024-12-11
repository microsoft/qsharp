namespace Kata.Verification {
    import Std.Convert.*;
    import KatasUtils.*;

    function FindFirstDiff(bits1 : Bool[], bits2 : Bool[]) : Int {
        for i in 0..Length(bits1) - 1 {
            if bits1[i] != bits2[i] {
                return i;
            }
        }
        return -1;
    }

    operation TwoBitstringSuperposition_Reference(qs : Qubit[], bits1 : Bool[], bits2 : Bool[]) : Unit is Adj + Ctl {
        // find the index of the first bit at which the bit strings are different
        let firstDiff = FindFirstDiff(bits1, bits2);

        // Hadamard corresponding qubit to create superposition
        H(qs[firstDiff]);

        // iterate through the bit strings again setting the final state of qubits
        for i in 0..Length(qs) - 1 {
            if bits1[i] == bits2[i] {
                // if two bits are the same, apply X or nothing
                if bits1[i] {
                    X(qs[i]);
                }
            } else {
                // if two bits are different, set their difference using CNOT
                if i > firstDiff {
                    CNOT(qs[firstDiff], qs[i]);
                    if bits1[i] != bits1[firstDiff] {
                        X(qs[i]);
                    }
                }
            }
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let bitstrings_one = [
            [false],
            [true],
            [false, false],
            [true, false],
            [false, false],
            [false, false, false],
            [false, false, false]
        ];

        let bitstrings_two = [
            [true],
            [false],
            [false, true],
            [false, true],
            [true, true],
            [false, false, true],
            [true, false, true]
        ];

        for i in 0..Length(bitstrings_one) - 1 {
            let bits1 = bitstrings_one[i];
            let bits2 = bitstrings_two[i];
            Message($"Testing for bits1 = {bits1} and bits2 = {bits2}...");
            if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
                Kata.TwoBitstringSuperposition(_, bits1, bits2),
                TwoBitstringSuperposition_Reference(_, bits1, bits2),
                Length(bits1)
            ) {
                return false;
            }
        }

        true
    }
}
