namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Random;

    operation DistinguishStates_MultiQubit_Threshold (Nqubit : Int, Nstate : Int, threshold : Double, statePrep : ((Qubit, Int) => Unit), testImpl : (Qubit => Bool)) : Bool {
        let nTotal = 1000;
        mutable nOk = 0;

        use qs = Qubit[Nqubit];
        for i in 1 .. nTotal {
            // get a random integer to define the state of the qubits
            let state = DrawRandomInt(0, Nstate - 1);

            // do state prep: convert |0⟩ to outcome with return equal to state
            statePrep(qs[0], state);

            // get the solution's answer and verify that it's a match
            let ans = testImpl(qs[0]);
            if ans == (state == 0) {
                set nOk += 1;
            }

            // we're not checking the state of the qubit after the operation
            ResetAll(qs);
        }

        if IntAsDouble(nOk) < threshold * IntAsDouble(nTotal) {
            Message($"{nTotal - nOk} test runs out of {nTotal} returned incorrect state which does not meet the required threshold of at least {threshold * 100.0}%.");
            Message("Incorrect.");
            return false;
        } else {
            Message("Correct!");
            return true;
        }
    }
}
