namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Canon;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Measurement;
    open Microsoft.Quantum.Random;

    // ------------------------------------------------------
    // "Framework" operation for testing multi-qubit tasks for distinguishing states of an array of qubits
    // with Int return
    operation DistinguishStates_MultiQubit(
        nQubits: Int,
        nStates: Int,
        statePrep: ((Qubit[], Int, Double) => Unit is Adj),
        testImpl: (Qubit[] => Int),
        preserveState: Bool,
        stateNames: String[]): Bool {

        let nTotal = 100;
        // misclassifications will store the number of times state i has been classified as state j (dimension nStates^2)
        mutable misclassifications = [0, size = nStates * nStates];
        // unknownClassifications will store the number of times state i has been classified as some invalid state (index < 0 or >= nStates)
        mutable unknownClassifications = [0, size = nStates];
                
        use qs = Qubit[nQubits];
        for i in 1 .. nTotal {
            // get a random integer to define the state of the qubits
            let state = DrawRandomInt(0, nStates - 1);
            // get a random rotation angle to define the exact state of the qubits
            // for some exercises, this value might be a dummy variable which does not matter
            let alpha = DrawRandomDouble(0.0, 1.0) * PI();
                
            // do state prep: convert |0...0⟩ to outcome with return equal to state
            statePrep(qs, state, alpha);

            // get the solution's answer and verify that it's a match, if not, increase the exact mismatch count
            let ans = testImpl(qs);
            if ((ans >= 0) and (ans < nStates)) {
                // classification result is a valid state index - check if is it correct
                if ans != state {
                    set misclassifications w/= ((state * nStates) + ans) <- (misclassifications[(state * nStates) + ans] + 1);
                }
            }
            else {
                // classification result is an invalid state index - file it separately
                set unknownClassifications w/= state <- (unknownClassifications[state] + 1);  
            }

            if preserveState {
                // check that the state of the qubit after the operation is unchanged
                Adjoint statePrep(qs, state, alpha);
                AssertAllZero(qs);
            } else {
                // we're not checking the state of the qubit after the operation
                ResetAll(qs);
            }
        }
        
        mutable totalMisclassifications = 0;
        for i in 0 .. nStates - 1 {
            for j in 0 .. nStates - 1 {
                if misclassifications[(i * nStates) + j] != 0 {
                    set totalMisclassifications += misclassifications[i * nStates + j];
                    Message($"Misclassified {stateNames[i]} as {stateNames[j]} in {misclassifications[(i * nStates) + j]} test runs.");
                }
            }
            if unknownClassifications[i] != 0 {
                set totalMisclassifications += unknownClassifications[i];
                Message($"Misclassified {stateNames[i]} as Unknown State in {unknownClassifications[i]} test runs.");
            }
        }
        return totalMisclassifications == 0;
    }
}
