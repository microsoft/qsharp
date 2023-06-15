    // ------------------------------------------------------
    // Exercise 6. Distinguish states |A❭ and |B❭
    // ------------------------------------------------------
    
    // |A⟩ =   cos(alpha) * |0⟩ - i sin(alpha) * |1⟩,
    // |B⟩ = - i sin(alpha) * |0⟩ + cos(alpha) * |1⟩.
    operation StatePrep_IsQubitA (alpha : Double, q : Qubit, state : Int) : Unit is Adj {
        if state == 0 {
            // convert |0⟩ to |B⟩
            X(q);
            Rx(2.0 * alpha, q);
        } else {
            // convert |0⟩ to |A⟩
            Rx(2.0 * alpha, q);
        }
    }

    
    @Test("QuantumSimulator")
    operation T6_IsQubitA () : Unit {
        for i in 0 .. 10 {
            let alpha = (PI() * IntAsDouble(i)) / 10.0;
            DistinguishTwoStates(StatePrep_IsQubitA(alpha, _, _), IsQubitA(alpha, _), 
                [$"|B⟩ = -i sin({i}π/10)|0⟩ + cos({i}π/10)|1⟩", $"|A⟩ = cos({i}π/10)|0⟩ + i sin({i}π/10)|1⟩"], false);
        }
    }
