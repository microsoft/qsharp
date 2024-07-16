namespace Kata {
    function FindFirstDiff (bits1 : Bool[], bits2 : Bool[]) : Int {
        for i in 0 .. Length(bits1) - 1 {
            if bits1[i] != bits2[i] {
                return i;
            }
        }
        return -1;
    }

    operation TwoBitstringSuperposition (qs : Qubit[], bits1 : Bool[], bits2 : Bool[]) : Unit is Adj + Ctl {
        // find the index of the first bit at which the bit strings are different
        let firstDiff = FindFirstDiff(bits1, bits2);

        // Hadamard corresponding qubit to create superposition
        H(qs[firstDiff]);

        // iterate through the bit strings again setting the final state of qubits
        for i in 0 .. Length(qs) - 1 {
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
}
