// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Intrinsic {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Core;
    open Microsoft.Quantum.Math;

    internal operation CH(control : Qubit, target : Qubit) : Unit is Adj {
        within {
            S(target);
            H(target);
            T(target);
        }
        apply {
            CNOT(control, target);
        }
    }

    internal operation CCH(control1 : Qubit, control2 : Qubit, target : Qubit) : Unit is Adj {
        within {
            S(target);
            H(target);
            T(target);
        }
        apply {
            CCNOT(control1, control2, target);
        }
    }

    internal operation ApplyGlobalPhase(theta : Double) : Unit is Ctl + Adj {
        body ... {}
        controlled (ctls, ...) {
            if Length(ctls) == 0 {
                // Noop
            }
            elif Length(ctls) == 1 {
                Rz(theta, ctls[0]);
            }
            else {
                Controlled R1(ctls[1..(Length(ctls) - 1)], (theta, ctls[0]));
            }
        }
    }

    internal operation CR1(theta : Double, control : Qubit, target : Qubit) : Unit is Adj {
        Rz(theta/2.0, target);
        Rz(theta/2.0, control);
        CNOT(control,target);
        Rz(-theta/2.0, target);
        CNOT(control,target);
    }

    internal operation CRz(control : Qubit, theta : Double, target : Qubit) : Unit is Adj {
        Rz(theta / 2.0, target);
        CNOT(control, target);
        Rz(-theta / 2.0, target);
        CNOT(control, target);
    }

    internal operation CS(control : Qubit, target : Qubit) : Unit is Adj + Ctl {
        T(control);
        T(target);
        CNOT(control, target);
        Adjoint T(target);
        CNOT(control, target);
    }

    internal operation CT(control : Qubit, target : Qubit) : Unit is Adj {
        let angle = PI() / 8.0;
        Rz(angle, control);
        Rz(angle, target);
        CNOT(control, target);
        Adjoint Rz(angle, target);
        CNOT(control, target);
    }

    internal operation MapPauli(qubit : Qubit, from : Pauli, to : Pauli) : Unit is Adj {
        if from == to {
        }
        elif (from == PauliZ and to == PauliX) or (from == PauliX and to == PauliZ) {
            H(qubit);
        }
        elif from == PauliZ and to == PauliY {
            H(qubit);
            S(qubit);
            H(qubit);
        }
        elif from == PauliY and to == PauliZ {
            H(qubit);
            Adjoint S(qubit);
            H(qubit);
        }
        elif from == PauliY and to == PauliX {
            S(qubit);
        }
        elif from == PauliX and to == PauliY {
            Adjoint S(qubit);
        }
        else {
            fail "Unsupported input";
        }
    }

    internal operation EntangleForJointMeasure(basis : Pauli, aux : Qubit, qubit : Qubit) : Unit {
        if basis == PauliX {
            Controlled X([aux], qubit);
        }
        elif basis == PauliZ {
            Controlled Z([aux], qubit);
        }
        elif basis == PauliY {
            Controlled Y([aux], qubit);
        }
    }

    /// Collects the given list of control qubits into one or two of the given auxiliarly qubits, using
    /// all but the last qubits in the auxiliary list as scratch qubits. The auxiliary list must be
    /// big enough to accomodate the data, so it is usually smaller than controls list by number of
    /// qubits needed for the eventual controlled unitary application. The passed adjustment value is
    /// used to ensure the right number of auxiliary qubits are processed.
    ///
    /// For example, if the controls list is 6 qubits, the auxiliary list must be 5 qubits, and the
    /// state from the 6 control qubits will be collected into the last qubit of the auxiliary array.
    internal operation CollectControls(ctls : Qubit[], aux : Qubit[], adjustment : Int) : Unit is Adj {
        // First collect the controls into the first part of the auxiliary list.
        for i in 0..2..(Length(ctls) - 2) {
            PhaseCCX(ctls[i], ctls[i + 1], aux[i / 2]);
        }
        // Then collect the auxiliary qubits in the first part of the list forward into the last
        // qubit of the auxiliary list. The adjustment is used to allow the caller to reduce or increase
        // the number of times this is run based on the eventual number of control qubits needed.
        for i in 0..((Length(ctls) / 2) - 2 - adjustment) {
            PhaseCCX(aux[i * 2], aux[(i * 2) + 1], aux[i + Length(ctls) / 2]);
        }
    }

    /// When collecting controls, if there is an uneven number of original control qubits then the
    /// last control and the second to last auxiliary will be collected into the last auxiliary.
    internal operation AdjustForSingleControl(ctls : Qubit[], aux : Qubit[]) : Unit is Adj {
        if Length(ctls) % 2 != 0 {
            PhaseCCX(ctls[Length(ctls) - 1], aux[Length(ctls) - 3], aux[Length(ctls) - 2]);
        }
    }

    internal operation PhaseCCX(control1 : Qubit, control2 : Qubit, target : Qubit) : Unit is Adj {
        // https://arxiv.org/pdf/1210.0974.pdf#page=2
        H(target);
        CNOT(target,control1);
        CNOT(control1,control2);
        T(control2);
        Adjoint T(control1);
        T(target);
        CNOT(target,control1);
        CNOT(control1,control2);
        Adjoint T(control2);
        CNOT(target,control2);
        H(target);
    }

    internal operation CCZ(control1 : Qubit, control2 : Qubit, target : Qubit) : Unit is Adj {
        within {
            H(target);
        } apply {
            CCNOT(control1, control2, target);
        }
    }

    internal operation CCY(control1 : Qubit, control2 : Qubit, target : Qubit) : Unit is Adj {
        within {
            MapPauli(target, PauliX, PauliY);
        }
        apply {
            CCNOT(control1, control2, target);
        }
    }

    internal operation CRxx(control : Qubit, theta : Double, qubit0 : Qubit, qubit1 : Qubit) : Unit {
        within {
            CNOT(qubit1, qubit0);
        }
        apply {
            Controlled Rx([control], (theta, qubit0));
        }
    }

    internal operation CRyy(control : Qubit, theta : Double, qubit0 : Qubit, qubit1 : Qubit) : Unit {
        within {
            CNOT(qubit1, qubit0);
        }
        apply {
            Controlled Ry([control], (theta, qubit0));
        }
    }

    internal operation CRzz(control : Qubit, theta : Double, qubit0 : Qubit, qubit1 : Qubit) : Unit {
        within {
            CNOT(qubit1, qubit0);
        }
        apply {
            Controlled Rz([control], (theta, qubit0));
        }
    }

    internal function IndicesOfNonIdentity (paulies : Pauli[]) : Int[] {
        mutable indices = [];
        for i in 0 .. Length(paulies) - 1 {
            if (paulies[i] != PauliI) {
                set indices += [i];
            }
        }
        indices
    }

    internal function RemovePauliI (paulis : Pauli[], qubits : Qubit[]) : (Pauli[], Qubit[]) {
        let indices = IndicesOfNonIdentity(paulis);
        let newPaulis = Subarray(indices, paulis);
        let newQubits = Subarray(indices, qubits);
        return (newPaulis, newQubits);
    }

    internal operation SpreadZ (from : Qubit, to : Qubit[]) : Unit is Adj {
        if (Length(to) > 0) {
            if (Length(to) > 1) {
                let half = Length(to) / 2;
                SpreadZ(to[0], to[half + 1 .. Length(to) - 1]);
                SpreadZ(from, to[1 .. half]);
            }
            CNOT(to[0], from);
        }
    }
}
