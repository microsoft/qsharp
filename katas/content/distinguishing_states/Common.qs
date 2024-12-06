namespace Kata.Verification {
    import Std.Arrays.*;
    import Std.Convert.*;
    import Std.Diagnostics.*;
    import Std.Math.*;

    operation StatePrep_BasisStateMeasurement(
        qs : Qubit[],
        state : Int,
        dummyVar : Double
    ) : Unit is Adj {
        if state / 2 == 1 {
            // |10⟩ or |11⟩
            X(qs[0]);
        }

        if state % 2 == 1 {
            // |01⟩ or |11⟩
            X(qs[1]);
        }
    }

    operation WState_Arbitrary_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        let N = Length(qs);

        if N == 1 {
            // base case of recursion: |1⟩
            X(qs[0]);
        } else {
            // |W_N⟩ = |0⟩|W_(N-1)⟩ + |1⟩|0...0⟩
            // do a rotation on the first qubit to split it into |0⟩ and |1⟩ with proper weights
            // |0⟩ -> sqrt((N-1)/N) |0⟩ + 1/sqrt(N) |1⟩
            let theta = ArcSin(1.0 / Sqrt(IntAsDouble(N)));
            Ry(2.0 * theta, qs[0]);

            // do a zero-controlled W-state generation for qubits 1..N-1
            X(qs[0]);
            Controlled WState_Arbitrary_Reference(qs[0..0], qs[1..N - 1]);
            X(qs[0]);
        }
    }

    function StatePrep_FindFirstDiff(
        bits1 : Bool[],
        bits2 : Bool[]
    ) : Int {
        for i in 0..Length(bits1) - 1 {
            if bits1[i] != bits2[i] {
                return i;
            }
        }

        return -1;
    }

    operation StatePrep_SuperpositionMeasurement(
        qs : Qubit[],
        bits1 : Bool[][],
        bits2 : Bool[][],
        state : Int,
        dummyVar : Double
    ) : Unit is Adj {
        let bits = state == 0 ? bits1 | bits2;
        StatePrep_BitstringSuperposition(qs, bits);
    }

    // A combination of tasks 14 and 15 from the Superposition kata
    operation StatePrep_BitstringSuperposition(
        qs : Qubit[],
        bits : Bool[][]
    ) : Unit is Adj + Ctl {
        let L = Length(bits);
        Fact(L == 1 or L == 2 or L == 4, "State preparation only supports arrays of 1, 2 or 4 bit strings.");
        if L == 1 {
            for i in 0..Length(qs) - 1 {
                if bits[0][i] {
                    X(qs[i]);
                }
            }
        }
        if L == 2 {
            // find the index of the first bit at which the bit strings are different
            let firstDiff = StatePrep_FindFirstDiff(bits[0], bits[1]);

            // Hadamard corresponding qubit to create superposition
            H(qs[firstDiff]);

            // iterate through the bit strings again setting the final state of qubits
            for i in 0..Length(qs) - 1 {
                if bits[0][i] == bits[1][i] {
                    // if two bits are the same, apply X or nothing
                    if bits[0][i] {
                        X(qs[i]);
                    }
                } else {
                    // if two bits are different, set their difference using CNOT
                    if i > firstDiff {
                        CNOT(qs[firstDiff], qs[i]);
                        if bits[0][i] != bits[0][firstDiff] {
                            X(qs[i]);
                        }
                    }
                }
            }
        }
        if L == 4 {
            let N = Length(qs);

            use anc = Qubit[2];
            // Put two ancillas into equal superposition of 2-qubit basis states
            ApplyToEachCA(H, anc);

            // Set up the right pattern on the main qubits with control on ancillas
            for i in 0..3 {
                for j in 0..N - 1 {
                    if bits[i][j] {
                        ApplyControlledOnInt(i, X, anc, qs[j]);
                    }
                }
            }

            // Uncompute the ancillas, using patterns on main qubits as control
            for i in 0..3 {
                if i % 2 == 1 {
                    ApplyControlledOnBitString(bits[i], X, qs, anc[0]);
                }
                if i / 2 == 1 {
                    ApplyControlledOnBitString(bits[i], X, qs, anc[1]);
                }
            }
        }
    }
}
