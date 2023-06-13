namespace Kata.Reference {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;

    // "Framework" operation for testing single-qubit tasks for distinguishing states of one qubit
    // with Bool return
    operation DistinguishTwoStates (statePrep : ((Qubit, Int) => Unit is Adj), testImpl : (Qubit => Bool), stateName : String[], checkFinalState : Bool) : Unit {
        let nTotal = 100;
        let nStates = 2;
        mutable misclassifications = [0, size = nStates];
        
        use q = Qubit();
        for i in 1 .. nTotal {
            // get a random bit to define whether qubit will be in a state corresponding to true return (1) or to false one (0)
            // state = 0 false return
            // state = 1 true return
            let state = DrawRandomInt(0, 1);

            // do state prep: convert |0⟩ to outcome with false return or to outcome with true return depending on state
            statePrep(q, state);

            // get the solution's answer and verify if NOT a match, then differentiate what kind of mismatch
            let ans = testImpl(q);
            if ans != (state == 1) {
                set misclassifications w/= state <- misclassifications[state] + 1;
            }
                
            // If the final state is to be verified, check if it matches the measurement outcome
            if checkFinalState {
                Adjoint statePrep(q, state);
                AssertQubit(Zero, q);
            } else {
                Reset(q);
            }
        }
        
        mutable totalMisclassifications = 0;
        for i in 0 .. nStates - 1 {
            if misclassifications[i] != 0 {
                set totalMisclassifications += misclassifications[i];
                Message($"Misclassified {stateName[i]} as {stateName[1 - i]} in {misclassifications[i]} test runs.");   
            }
        }
        
        // This check will tell the total number of failed classifications
        Fact(totalMisclassifications == 0, $"{totalMisclassifications} test runs out of {nTotal} returned incorrect state (see output for details).");
    }

}
