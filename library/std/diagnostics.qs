// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Diagnostics {
    open QIR.Intrinsic;

    function DumpMachine() : Unit {
        body intrinsic;
    }

    @Config(Full)
    operation CheckZero(qubit : Qubit) : Bool {
        body intrinsic;
    }

    @Config(Full)
    operation CheckAllZero(qubits : Qubit[]) : Bool {
        for q in qubits {
            if not CheckZero(q) {
                return false;
            }
        }

        return true;
    }

    /// Checks whether a classical condition is true, and throws an exception if it is not.
    function Fact(actual : Bool, message : String) : Unit {
        if (not actual) {
            fail message;
        }
    }

    /// # Summary
    /// Given two registers, prepares the maximally entangled state
    /// between each pair of qubits on the respective registers.
    /// All qubits must start in the |0⟩ state.
    ///
    /// # Input
    /// ## left
    /// A qubit array in the |0...0⟩ state
    /// ## right
    /// A qubit array in the |0...0⟩ state
    internal operation PrepareEntangledState (left : Qubit[], right : Qubit[]) : Unit
    is Adj + Ctl {
        Fact( Length(left) == Length(right), "PrepareEntangledState: Lengths of qubits registers must be equal.");
        for idxQubit in 0 .. Length(left) - 1  {
            H(left[idxQubit]);
            CNOT(left[idxQubit], right[idxQubit]);
        }
    }
    
    /// # Summary
    /// Given two operations, asserts that they act identically for all input states.
    ///
    /// # Description
    /// This assertion is implemented by using the Choi–Jamiołkowski isomorphism to reduce
    /// the assertion to one of a qubit state assertion on two entangled registers.
    /// Thus, this operation needs only a single call to each operation being tested,
    /// but requires twice as many qubits to be allocated.
    /// This assertion can be used to ensure, for instance, that an optimized version of an
    /// operation acts identically to its naïve implementation, or that an operation
    /// which acts on a range of non-quantum inputs agrees with known cases.
    ///
    /// # Remarks
    /// This operation requires that the operation modeling the expected behavior is
    /// adjointable, so that the inverse can be performed on the target register alone.
    /// Formally, one can specify a transpose operation, which relaxes this requirement,
    /// but the transpose operation is not in general physically realizable for arbitrary
    /// quantum operations and thus is not included here as an option.
    ///
    /// # Input
    /// ## nQubits
    /// Number of qubits to pass to each operation.
    /// ## actual
    /// Operation to be tested.
    /// ## expected
    /// Operation defining the expected behavior for the operation under test.
    /// # Output
    /// True if operations are equal, false otherwise.
    @Config(Full)
    operation CheckOperationsAreEqual (
        nQubits : Int,
        actual : (Qubit[] => Unit),
        expected : (Qubit[] => Unit is Adj)) : Bool {

        // Prepare a reference register entangled with the target register.
        use reference = Qubit[nQubits];
        use target = Qubit[nQubits];

        // Apply operations.
        within {
            PrepareEntangledState(reference, target);
        } apply {
            actual(target);
            Adjoint expected(target);
        }

        // Check and resturn result.
        let areEqual = CheckAllZero(reference + target);
        ResetAll(target);
        ResetAll(reference);
        areEqual
    }


}
