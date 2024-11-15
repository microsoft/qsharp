// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Katas {
    import KatasUtils.*;
    export
        CheckOperationsAreEqualStrict,
        CheckOperationsEquivalenceOnZeroState,
        CheckOperationsEquivalenceOnInitialStateStrict,
        CheckOperationsEquivalenceOnZeroStateStrict,
        ShowQuantumStateComparison,
        CheckOperationsEquivalenceOnZeroStateWithFeedback,
        EntangleRegisters,
        PrepDemoState,
        DistinguishTwoStates_SingleQubit,
        DistinguishStates_MultiQubit,
        BoolArrayAsKetState,
        IntArrayAsStateName,
        CheckOracleImplementsFunction;
}

namespace KatasUtils {
    import Std.Arrays.*;
    import Std.Convert.*;
    import Std.Diagnostics.*;
    import Std.Math.*;
    import Std.Random.*;

    /// # Summary
    /// Given two operations, checks whether they act identically (including global phase) for all input states.
    /// This is done through controlled versions of the operations instead of plain ones which convert the global phase
    /// into a relative phase that can be detected.
    operation CheckOperationsAreEqualStrict(
        inputSize : Int,
        op : (Qubit[] => Unit is Adj + Ctl),
        reference : (Qubit[] => Unit is Adj + Ctl)
    ) : Bool {
        Fact(inputSize > 0, "`inputSize` must be positive");
        let controlledOp = register => Controlled op(register[...0], register[1...]);
        let controlledReference = register => Controlled reference(register[...0], register[1...]);
        let areEquivalent = CheckOperationsAreEqual(inputSize + 1, controlledOp, controlledReference);
        areEquivalent
    }

    /// # Summary
    /// Given two operations, checks whether they act identically on the zero state |0〉 ⊗ |0〉 ⊗ ... ⊗ |0〉 composed of
    /// `inputSize` qubits.
    operation CheckOperationsEquivalenceOnZeroState(
        op : (Qubit[] => Unit),
        reference : (Qubit[] => Unit is Adj),
        inputSize : Int
    ) : Bool {
        Fact(inputSize > 0, "`inputSize` must be positive");
        use target = Qubit[inputSize];
        op(target);
        Adjoint reference(target);
        let isCorrect = CheckAllZero(target);
        ResetAll(target);
        isCorrect
    }


    /// # Summary
    /// Given two operations, checks whether they act identically on the given initial state composed of `inputSize` qubits.
    /// The initial state is prepared by applying the `initialState` operation to the state |0〉 ⊗ |0〉 ⊗ ... ⊗ |0〉.
    /// This operation introduces a control qubit to convert a global phase into a relative phase to be able to detect it.
    /// `initialState` operation should be deterministic.
    operation CheckOperationsEquivalenceOnInitialStateStrict(
        initialState : Qubit[] => Unit is Adj,
        op : (Qubit[] => Unit is Adj + Ctl),
        reference : (Qubit[] => Unit is Adj + Ctl),
        inputSize : Int
    ) : Bool {
        use (control, target) = (Qubit(), Qubit[inputSize]);
        within {
            H(control);
            initialState(target);
        } apply {
            Controlled op([control], target);
            Adjoint Controlled reference([control], target);
        }

        let isCorrect = CheckAllZero([control] + target);
        ResetAll([control] + target);
        isCorrect
    }


    /// # Summary
    /// Given two operations, checks whether they act identically on the zero state |0〉 ⊗ |0〉 ⊗ ... ⊗ |0〉 composed of
    /// `inputSize` qubits.
    /// This operation introduces a control qubit to convert a global phase into a relative phase to be able to detect
    /// it.
    operation CheckOperationsEquivalenceOnZeroStateStrict(
        op : (Qubit[] => Unit is Adj + Ctl),
        reference : (Qubit[] => Unit is Adj + Ctl),
        inputSize : Int
    ) : Bool {
        Fact(inputSize > 0, "`inputSize` must be positive");
        CheckOperationsEquivalenceOnInitialStateStrict(qs => (), op, reference, inputSize)
    }


    /// # Summary
    /// Shows the comparison of the quantum states produced by a specific operation and a reference operation
    /// when applied to the state prepared using deterministic operation `initialState`.
    operation ShowQuantumStateComparison(
        registerSize : Int,
        initialState : Qubit[] => Unit,
        op : Qubit[] => Unit,
        reference : Qubit[] => Unit
    ) : Unit {
        {
            use register = Qubit[registerSize];
            initialState(register);

            Message("Initial quantum state:");
            DumpMachine();

            // Apply the reference operation and dump the simulator state
            reference(register);
            Message("Expected quantum state after applying the operation:");
            DumpMachine();
            ResetAll(register);
        }

        {
            use register = Qubit[registerSize];
            initialState(register);
            // Apply the comparison operation and dump the simulator state
            op(register);
            Message("Actual quantum state after applying the operation:");
            DumpMachine();
            ResetAll(register);
        }
    }

    /// # Summary
    /// Given two operations, checks whether they act identically on the zero state |0〉 ⊗ |0〉 ⊗ ... ⊗ |0〉 composed of
    /// `inputSize` qubits. If they don't, prints user feedback.
    operation CheckOperationsEquivalenceOnZeroStateWithFeedback(
        testImpl : (Qubit[] => Unit),
        refImpl : (Qubit[] => Unit is Adj),
        inputSize : Int
    ) : Bool {

        let isCorrect = CheckOperationsEquivalenceOnZeroState(testImpl, refImpl, inputSize);

        // Output different feedback to the user depending on whether the exercise was correct.
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
            ShowQuantumStateComparison(inputSize, (qs => ()), testImpl, refImpl);
        }
        isCorrect
    }


    internal operation EntangleRegisters(
        control : Qubit[],
        target : Qubit[]
    ) : Unit is Adj + Ctl {
        Fact(
            Length(control) == Length(target),
            $"The length of qubit registers must be the same."
        );

        for index in IndexRange(control) {
            H(control[index]);
            CNOT(control[index], target[index]);
        }
    }


    /// # Summary
    /// Prepare a random uneven superposition state on the given qubit array.
    operation PrepDemoState(qs : Qubit[]) : Unit {
        Fact(Length(qs) <= 4, "States with 5 qubits or more are not supported.");
        let probs = [0.36, 0.25, 1. / 3., 1. / 5.][...Length(qs) - 1];
        for (q, prob) in Zipped(qs, probs) {
            Ry(ArcCos(Sqrt(prob)) * 2.0, q);
        }
    }


    // "Framework" operation for testing single-qubit tasks for distinguishing states of one qubit
    // with Bool return
    operation DistinguishTwoStates_SingleQubit(
        statePrep : ((Qubit, Int) => Unit is Adj),
        testImpl : (Qubit => Bool),
        stateNames : String[],
        preserveState : Bool
    ) : Bool {

        let nTotal = 100;
        let nStates = 2;
        mutable misclassifications = [0, size = nStates];

        use q = Qubit();
        for _ in 1..nTotal {
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

            // If the final state is to be preserved, check if it was not modified
            if preserveState {
                Adjoint statePrep(q, state);
                if not CheckZero(q) {
                    Message($"Input quantum state {stateNames[state]} was not preserved during the measurement.");
                    Reset(q);
                    return false;
                }
            } else {
                Reset(q);
            }
        }

        mutable totalMisclassifications = 0;
        for i in 0..nStates - 1 {
            if misclassifications[i] != 0 {
                set totalMisclassifications += misclassifications[i];
                Message($"Misclassified {stateNames[i]} as {stateNames[1 - i]} in {misclassifications[i]} test runs.");
            }
        }

        totalMisclassifications == 0
    }


    // "Framework" operation for testing multi-qubit tasks for distinguishing states of an array of qubits
    // with Int return
    operation DistinguishStates_MultiQubit(
        nQubits : Int,
        nStates : Int,
        statePrep : ((Qubit[], Int, Double) => Unit is Adj),
        testImpl : (Qubit[] => Int),
        preserveState : Bool,
        stateNames : String[]
    ) : Bool {

        let nTotal = 100;
        // misclassifications will store the number of times state i has been classified as state j (dimension nStates^2)
        mutable misclassifications = [0, size = nStates * nStates];
        // unknownClassifications will store the number of times state i has been classified as some invalid state (index < 0 or >= nStates)
        mutable unknownClassifications = [0, size = nStates];

        use qs = Qubit[nQubits];
        for _ in 1..nTotal {
            // get a random integer to define the state of the qubits
            let state = DrawRandomInt(0, nStates - 1);
            // get a random rotation angle to define the exact state of the qubits
            // for some exercises, this value might be a dummy variable which does not matter
            let alpha = DrawRandomDouble(0.0, 1.0) * PI();

            // do state prep: convert |0...0⟩ to outcome with return equal to state
            statePrep(qs, state, alpha);

            // get the solution's answer and verify that it's a match, if not, increase the exact mismatch count
            let ans = testImpl(qs);
            if ans >= 0 and ans < nStates {
                // classification result is a valid state index - check if is it correct
                if ans != state {
                    set misclassifications w/= ((state * nStates) + ans) <- (misclassifications[(state * nStates) + ans] + 1);
                }
            } else {
                // classification result is an invalid state index - file it separately
                set unknownClassifications w/= state <- (unknownClassifications[state] + 1);
            }

            if preserveState {
                // check that the state of the qubit after the operation is unchanged
                Adjoint statePrep(qs, state, alpha);
                if not CheckAllZero(qs) {
                    Message($"Input quantum state {stateNames[state]} was not preserved during the measurement.");
                    ResetAll(qs);
                    return false;
                }
            } else {
                // we're not checking the state of the qubit after the operation
                ResetAll(qs);
            }
        }

        mutable totalMisclassifications = 0;
        for i in 0..nStates - 1 {
            for j in 0..nStates - 1 {
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
        totalMisclassifications == 0
    }

    // Helper function to convert a boolean array to its ket state representation
    function BoolArrayAsKetState(bits : Bool[]) : String {
        mutable stateName = "|";
        for i in 0..Length(bits) - 1 {
            set stateName += (bits[i] ? "1" | "0");
        }

        return stateName + "⟩";
    }

    // Helper function to convert an array of bit strings to its ket state representation
    function IntArrayAsStateName(
        qubits : Int,
        bitStrings : Bool[][]
    ) : String {
        mutable statename = "";
        for i in 0..Length(bitStrings) - 1 {
            if i > 0 {
                set statename += " + ";
            }
            set statename += BoolArrayAsKetState(bitStrings[i]);
        }

        return statename;
    }

    /// # Summary
    /// Given a marking oracle acting on N inputs, and a classical function acting on N bits,
    /// checks whether the oracle effect matches that of the function on every classical input.
    operation CheckOracleImplementsFunction(
        N : Int,
        oracle : (Qubit[], Qubit) => Unit,
        f : Bool[] -> Bool
    ) : Bool {
        let size = 1 <<< N;
        use (input, target) = (Qubit[N], Qubit());
        for k in 0..size - 1 {
            // Prepare k-th bit vector
            let binaryLE = IntAsBoolArray(k, N);

            // "binary" is little-endian notation, so the second vector tried has qubit 0 in state 1 and the rest in state 0
            ApplyPauliFromBitString(PauliX, true, binaryLE, input);

            // Apply the operation
            oracle(input, target);

            // Calculate the expected classical result
            let val = f(binaryLE);

            // Apply operations that will revert the qubits to the 0 state if the oracle acted correctly.
            if val {
                X(target);
            }
            ApplyPauliFromBitString(PauliX, true, binaryLE, input);

            if not CheckAllZero(input + [target]) {
                Message($"Unexpected result on input {BoolArrayAsKetState(binaryLE)}.");
                if not CheckAllZero(input) {
                    Message("The state of the input qubits changed, or they ended up entangled with the target qubit.");
                    Message("The state of the system after oracle application:");
                    DumpMachine();
                } else {
                    Message($"Expected result `{val}`, got `{not val}`.");
                }
                ResetAll(input + [target]);
                return false;
            }
        }
        return true;
    }
}
