namespace Test {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Unstable.StatePreparation;


    operation TestPlusState(): Unit {
        use q = Qubit();
        PreparePureStateD([Sqrt(0.5), Sqrt(0.5)], [q]);
        // Ucompute plus
        H(q);
        if ( not CheckZero(q) ) {
            fail "|+> preparation failed.";
        }
    }

    operation TestMinusState(): Unit {
        use q = Qubit();
        PreparePureStateD([Sqrt(0.5), -Sqrt(0.5)], [q]);
        // Ucompute minus
        H(q);
        X(q);
        if ( not CheckZero(q) ) {
            fail "|-> preparation failed.";
        }
    }

    operation TestBellState(): Unit {
        use q = Qubit[2];
        PreparePureStateD([Sqrt(0.5), 0.0, 0.0, Sqrt(0.5)], q);
        // Ucompute Bell
        CNOT(q[0], q[1]);
        H(q[0]);
        if ( not CheckAllZero(q) ) {
            fail "Bell state preparation failed.";
        }
    }

    operation TestCat3State(): Unit {
        use q = Qubit[3];
        PreparePureStateD([Sqrt(0.5), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, Sqrt(0.5)], q);
        // Ucompute Cat
        CNOT(q[0], q[2]);
        CNOT(q[0], q[1]);
        H(q[0]);
        if ( not CheckAllZero(q) ) {
            fail "Cat state preparation failed.";
        }
    }

    operation PrepareComplex(qs : Qubit[]) : Unit is Adj {
        H(qs[0]);
        T(qs[0]);
        H(qs[1]);
        S(qs[1]);
    }

    operation TestPrepareComplex(): Unit {
        use q = Qubit[2];
        let c00 = ComplexPolar(0.5, 0.0);
        let c01 = ComplexPolar(0.5, PI()/4.0);
        let c10 = ComplexPolar(0.5, PI()/2.0);
        let c11 = ComplexPolar(0.5, 3.0*PI()/4.0);
        ApproximatelyPreparePureStateCP(0.0, [c00, c01, c10, c11], q);
        Adjoint PrepareComplex(q);
        if ( not CheckAllZero(q) ) {
            fail "Complex state preparation failed.";
        }
    }

    operation TestPreparationCompletion(): Unit {
        let testCases = [
            // Test positive coefficients
            [0.773761, 0.633478],
            [0.183017, 0.406973, 0.604925, 0.659502],
            [0.0986553, 0.359005, 0.465689, 0.467395, 0.419893, 0.118445, 0.461883, 0.149609],
            [0.271471, 0.0583654, 0.11639, 0.36112, 0.307383, 0.193371, 0.274151, 0.332542, 0.130172, 0.222546, 0.314879, 0.210704, 0.212429, 0.245518, 0.30666, 0.22773],

            // Test negative coefficients; should give same probabilities as positive coefficients
            [-0.773761, 0.633478],
            [0.183017, -0.406973, 0.604925, 0.659502],
            [0.0986553, -0.359005, 0.465689, -0.467395, 0.419893, 0.118445, -0.461883, 0.149609],
            [-0.271471, 0.0583654, 0.11639, 0.36112, -0.307383, 0.193371, -0.274151, 0.332542, 0.130172, 0.222546, 0.314879, -0.210704, 0.212429, 0.245518, -0.30666, -0.22773],

            // Test unnormalized coefficients
            [1.0986553, 0.359005, 0.465689, -0.467395, 0.419893, 0.118445, 0.461883, 0.149609],

            // Test missing coefficients
            [1.0986553, 0.359005, 0.465689, -0.467395, 0.419893, 0.118445]
        ];

        for coefficients in testCases {
            let L = Length(coefficients);
            let N = Ceiling(Log(IntAsDouble(L))/LogOf2() - 0.001);
            use q = Qubit[N];
            PreparePureStateD(coefficients, q);
            // NOTE: We cannot check that the state is actually correct.
            if CheckAllZero(q) {
                fail "Failed to prepare pure state. L={L}, N={N}, coefficients={coefficients}.";
            }
            Adjoint PreparePureStateD(coefficients, q);
            if not CheckAllZero(q) {
                fail "Failed to prepare and unprepare pure state. L={L}, N={N}, coefficients={coefficients}.";
            }
        }
    }


}
