namespace Kata.Verification {

    open Microsoft.Quantum.Diagnostics;

    // TODO: Move to KataLibrary.qs or even to the standard library.

    /// # Summary
    /// Given two registers, prepares the maximally entangled state 
    /// between each pair of qubits on the respective registers.
    /// All qubits must start in the |0⟩ state.
    ///
    /// The result is that corresponding pairs of qubits from each register are in the
    /// $\bra{\beta_{00}}\ket{\beta_{00}}$.
    ///
    /// # Input
    /// ## left
    /// A qubit array in the $\ket{0\cdots 0}$ state
    /// ## right
    /// A qubit array in the $\ket{0\cdots 0}$ state
    internal operation PrepareEntangledState (left : Qubit[], right : Qubit[]) : Unit
    is Adj + Ctl {

        for idxQubit in 0 .. Length(left) - 1
        {
            H(left[idxQubit]);
            Controlled X([left[idxQubit]], right[idxQubit]);
        }
    }
    
    
    /// # Summary
    /// Given two operations, asserts that they act identically for all input states.
    ///
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
    operation CheckOperationsEqualReferenced (nQubits : Int, actual : (Qubit[] => Unit), expected : (Qubit[] => Unit is Adj)) : Bool {
        // Prepare a reference register entangled with the target register.
        use (reference, target) = (Qubit[nQubits], Qubit[nQubits]) {
            PrepareEntangledState(reference, target);
            actual(target);
            Adjoint expected(target);
            Adjoint PrepareEntangledState(reference, target);
            let isCorrect = CheckAllZero(reference + target);
            ResetAll(target);
            ResetAll(reference);
            isCorrect
        }
    }

    // ------------------------------------------------------
    // Helper functions
    operation ApplyOracle (qs : Qubit[], oracle : ((Qubit[], Qubit) => Unit is Adj + Ctl)) : Unit is Adj + Ctl {
        let N = Length(qs);
        oracle(qs[0 .. N - 2], qs[N - 1]);
    }

    operation CheckTwoOraclesAreEqual (nQubits : Range,
        oracle1 : ((Qubit[], Qubit) => Unit is Adj + Ctl),
        oracle2 : ((Qubit[], Qubit) => Unit is Adj + Ctl)) : Bool {
        let sol = ApplyOracle(_, oracle1);
        let refSol = ApplyOracle(_, oracle2);

        for i in nQubits {
            if not CheckOperationsEqualReferenced(i + 1, sol, refSol) {
                return false;
            }
        }
        true
    }

    // TODO: Move to standard library

    /// # Summary
    /// Applies a unitary operation on the target register, controlled on a a state specified by a given
    /// bit mask.
    ///
    /// # Input
    /// ## bits
    /// The bit string to control the given unitary operation on.
    /// ## oracle
    /// The unitary operation to be applied on the target register.
    /// ## targetRegister
    /// The target register to be passed to `oracle` as an input.
    /// ## controlRegister
    /// A quantum register that controls application of `oracle`.
    ///
    /// # Remarks
    /// The pattern given by `bits` may be shorter than `controlRegister`,
    /// in which case additional control qubits are ignored (that is, neither
    /// controlled on $\ket{0}$ nor $\ket{1}$).
    /// If `bits` is longer than `controlRegister`, an error is raised.
    ///
    /// For example, `bits = [0,1,0,0,1]` means that `oracle` is applied if and only if `controlRegister`
    /// is in the state $\ket{0}\ket{1}\ket{0}\ket{0}\ket{1}$.
    operation ApplyControlledOnBitString<'T> (bits : Bool[], oracle : ('T => Unit is Adj + Ctl), controlRegister : Qubit[], targetRegister : 'T)
    : Unit is Adj + Ctl {
        // The control register must have enough bits to implement the requested control.
        Fact(Length(bits) <= Length(controlRegister), "Control register shorter than control pattern.");

        // Use a subregister of the controlled register when
        // bits is shorter than controlRegister.
        let controlSubregister = controlRegister[...Length(bits) - 1];
        within {
            ApplyPauliFromBitString(PauliX, false, bits, controlSubregister);
        } apply {
            Controlled oracle(controlSubregister, targetRegister);
        }
    }

}
