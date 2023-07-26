namespace Kata.Verification {

    // ------------------------------------------------------
    // Exercise 7: State selection using partial measurements
    // ------------------------------------------------------
    operation stateInitialize_StateSelction(alpha: Double, qs: Qubit[]): Unit {
        // Prepare the state to be input to the testImplementation
        // set the second qubit in a superposition a |0⟩ + b|1⟩
        // with a = cos alpha, b = sin alpha
        Ry(2.0 * alpha, qs[1]); 

        H(qs[0]);
        // Apply CX gate
        CX(qs[0], qs[1]);
    }

    operation statePrepare_StateSelction(alpha: Double, Choice: Int, qs: Qubit[]): Unit is Adj {
        // The expected state of the second qubit for the exercise.

        // set the second qubit in a superposition a |0⟩ + b|1⟩
        // with a = cos alpha, b = sin alpha
        Ry(2.0 * alpha, qs[1]); 
        if Choice == 1 { 
            // if the Choice is 1, change the state to b|0⟩ + a|1⟩
            X(qs[1]);
        }
    }


    @EntryPoint()
    operation CheckSolution(): Bool {
        use qs = Qubit[2];
        for i in 0 .. 5 {
            let alpha = (PI() * IntAsDouble(i)) / 5.0;
            
            //for Choice = 0 and 1,
            for Choice in 0 .. 1 {
                // Prepare the state to be input to the testImplementation
                stateInitialize_StateSelction(alpha, qs);

                // operate testImplementation
                Kata.StateSelction(qs, Choice);
                // reset the first qubit, since its state does not matter
                Reset(qs[0]);

                // apply adjoint reference operation and check that the result is correct
                Adjoint statePrepare_StateSelction(alpha, Choice, qs);
                
                if not CheckAllZero(qs) {
                    return false;
                }
                ResetAll(qs);
            }           
        }
        return true;
    }

}
