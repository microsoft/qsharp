import Std.OpenQASM.Intrinsic.__quantum__qis__barrier__body;
// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// This file contains 24 single-qubit Clifford transformations that remap Pauli axes on a Bloch sphere.
// These mappings are useful to apply operations in different Pauli bases.
// For more information, see https://wikipedia.org/wiki/Clifford_group

// For example, rotation around the X axis can be expressed as a rotation around the Z axis
// by remapping the Pauli axes so that X axis becomes Z axis. The following two operations are equivalent:
//     Rx(0.1, q)
// and
//     within {
//         RemapXYZAxisTo_Zxy(q);
//     } apply {
//         Rz(0.1, q);
//     }

// Naming convention for transformations:
// - RemapXYZAxisTo_αβγ
// where αβγ describes how Pauli axes XYZ are mapped.
// α is the new direction for former X, β is the new direction for former Y, and
// γ is the new direction for former Z. Capital letters indicate that positive direction
// is preserved, while lower case letters indicate that the direction is flipped.
// For example, RemapXYZAxisTo_Zxy transforms the Bloch sphere so that
// X becomes Z, Y becomes -X, and Z becomes -Y. All of these transformations are rotations.

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X remains X, Y becomes -Y, and Z becomes -Z. This is done by applying the X gate.
operation RemapXYZAxisTo_Xyz(q : Qubit) : Unit is Adj + Ctl {
    X(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X remains X, Y becomes Z, and Z becomes -Y. This is done by applying HSH gate sequence.
/// An alternative gate sequence is S⁻¹HS⁻¹.
operation RemapXYZAxisTo_XZy(q : Qubit) : Unit is Adj + Ctl {
    H(q);
    S(q);
    H(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes X, Y becomes -Z, and Z becomes Y. This is done by applying HS⁻¹H gate sequence.
/// An alternative gate sequence is SHS.
operation RemapXYZAxisTo_XzY(q : Qubit) : Unit is Adj + Ctl {
    H(q);
    Adjoint S(q);
    H(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes -X, Y remains Y, and Z becomes -Z. This is done by applying Y gate.
operation RemapXYZAxisTo_xYz(q : Qubit) : Unit is Adj + Ctl {
    Y(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes -X, Y becomes -Y, and Z remains Z. This is done by applying Z gate.
operation RemapXYZAxisTo_xyZ(q : Qubit) : Unit is Adj + Ctl {
    Z(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes -X, Y becomes Z, and Z becomes Y. This is done by applying S⁻¹HS gate sequence.
operation RemapXYZAxisTo_xZY(q : Qubit) : Unit is Adj + Ctl {
    Adjoint S(q);
    H(q);
    S(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes -X, Y becomes -Y, and Z becomes -Z. This is done by applying SHS⁻¹ gate sequence.
operation RemapXYZAxisTo_xzy(q : Qubit) : Unit is Adj + Ctl {
    S(q);
    H(q);
    Adjoint S(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes Y, Y remains Y, and Z becomes -Z. This is done by applying XS gate sequence.
/// Alternative gate sequences are YS⁻¹, SY, S⁻¹X.
operation RemapXYZAxisTo_YXz(q : Qubit) : Unit is Adj + Ctl {
    X(q);
    S(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes Y, Y becomes -X, and Z remains Z. This is done by applying S gate.
operation RemapXYZAxisTo_YxZ(q : Qubit) : Unit is Adj + Ctl {
    S(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes Y, Y becomes Z, and Z becomes X. This is done by applying S⁻¹H gate sequence.
operation RemapXYZAxisTo_YZX(q : Qubit) : Unit is Adj + Ctl {
    Adjoint S(q);
    H(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes Y, Y becomes -Z, and Z becomes -X. This is done by applying SXH gate sequence.
/// Alternative gate sequences are YSH, S⁻¹YH, S⁻¹HY, XS⁻¹H, SHZ.
operation RemapXYZAxisTo_Yzx(q : Qubit) : Unit is Adj + Ctl {
    S(q);
    X(q);
    H(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes -Y, Y becomes X, and Z remains Z. This is done by applying S⁻¹ gate.
operation RemapXYZAxisTo_yXZ(q : Qubit) : Unit is Adj + Ctl {
    Adjoint S(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes -Y, Y becomes -X, and Z becomes -Z. This is done by applying SX gate sequence.
/// Alternative gate sequences are YS, S⁻¹Y, XS⁻¹.
operation RemapXYZAxisTo_yxz(q : Qubit) : Unit is Adj + Ctl {
    S(q);
    X(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes -Y, Y becomes Z, and Z becomes -X. This is done by applying XSH gate sequence.
/// Alternative gate sequences are S⁻¹HZ, YS⁻¹H, SYH, S⁻¹XH, SHY.
operation RemapXYZAxisTo_yZx(q : Qubit) : Unit is Adj + Ctl {
    X(q);
    S(q);
    H(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes -Y, Y becomes -Z, and Z becomes X. This is done by applying SH gate sequence.
operation RemapXYZAxisTo_yzX(q : Qubit) : Unit is Adj + Ctl {
    S(q);
    H(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes Z, Y becomes X, and Z becomes Y. This is done by applying HS gate sequence.
operation RemapXYZAxisTo_ZXY(q : Qubit) : Unit is Adj + Ctl {
    H(q);
    S(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes Z, Y becomes -X, and Z becomes -Y. This is done by applying HS⁻¹ gate sequence.
operation RemapXYZAxisTo_Zxy(q : Qubit) : Unit is Adj + Ctl {
    H(q);
    Adjoint S(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes Z, Y remains Y, and Z becomes -X. This is done by applying XH gate sequence.
/// An alternative gate sequence is HZ.
operation RemapXYZAxisTo_ZYx(q : Qubit) : Unit is Adj + Ctl {
    X(q);
    H(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes Z, Y becomes -Y, and Z becomes X. This is done by applying H gate.
operation RemapXYZAxisTo_ZyX(q : Qubit) : Unit is Adj + Ctl {
    H(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes -Z, Y becomes X, and Z becomes -Y. This is done by applying HSX gate sequence.
/// Alternative gate sequences are YHS, HYS, ZHS⁻¹, HXS⁻¹, HS⁻¹Y.
operation RemapXYZAxisTo_zXy(q : Qubit) : Unit is Adj + Ctl {
    H(q);
    S(q);
    X(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes -Z, Y becomes -X, and Z becomes Y. This is done by applying HXS gate sequence.
/// Alternative gate sequences are HSY, YHS⁻¹, HYS⁻¹, ZHS, HXS, HS⁻¹X.
operation RemapXYZAxisTo_zxY(q : Qubit) : Unit is Adj + Ctl {
    H(q);
    X(q);
    S(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes -Z, Y remains Y, and Z becomes X. This is done by applying HX gate sequence.
/// An alternative gate sequence is ZH.
operation RemapXYZAxisTo_zYX(q : Qubit) : Unit is Adj + Ctl {
    H(q);
    X(q);
}

/// # Summary
/// Rotates a Bloch sphere so that Pauli axis are mapped as follows:
/// X becomes -Z, Y becomes -Y, and Z becomes -X. This is done by applying HY gate sequence.
/// An alternative gate sequence is YH.
operation RemapXYZAxisTo_zyx(q : Qubit) : Unit is Adj + Ctl {
    H(q);
    Y(q);
}

/// # Summary
/// Performs no rotations on a Bloch sphere so that all Pauli axis remain unchanged:
/// X remains X, Y remains Y, and Z remains Z. No gates are applied.
/// This operation is only present for completeness.
operation RemapXYZAxisTo_XYZ(q : Qubit) : Unit is Adj + Ctl {
    // Identity mapping, no gates applied.
}

/// # Summary
/// Maps a Pauli axis from one direction to another by applying an appropriate Clifford transformation.
/// TODO: Active transformation vs. passive transformation. https://wikipedia.org/wiki/Active_and_passive_transformation
operation MapPauliAxis(from : Pauli, to : Pauli, q : Qubit) : Unit is Adj + Ctl {
    if from == to {
        RemapXYZAxisTo_XYZ(q);
    } elif (from == PauliZ and to == PauliX) or (from == PauliX and to == PauliZ) {
        RemapXYZAxisTo_ZyX(q);
    } elif from == PauliZ and to == PauliY {
        RemapXYZAxisTo_YZX(q); // TODO: Likely also acceptable.
        //RemapXYZAxisTo_XZy(q);
    } elif from == PauliY and to == PauliZ {
        RemapXYZAxisTo_XzY(q);
    } elif from == PauliY and to == PauliX {
        RemapXYZAxisTo_YxZ(q);
    } elif from == PauliX and to == PauliY {
        RemapXYZAxisTo_yXZ(q);
    } else {
        fail "Unsupported mapping.";
    }
}

export
    RemapXYZAxisTo_Xyz,
    RemapXYZAxisTo_XZy,
    RemapXYZAxisTo_XzY,
    RemapXYZAxisTo_xYz,
    RemapXYZAxisTo_xyZ,
    RemapXYZAxisTo_xZY,
    RemapXYZAxisTo_xzy,
    RemapXYZAxisTo_YXz,
    RemapXYZAxisTo_YxZ,
    RemapXYZAxisTo_YZX,
    RemapXYZAxisTo_Yzx,
    RemapXYZAxisTo_yXZ,
    RemapXYZAxisTo_yxz,
    RemapXYZAxisTo_yZx,
    RemapXYZAxisTo_yzX,
    RemapXYZAxisTo_ZXY,
    RemapXYZAxisTo_Zxy,
    RemapXYZAxisTo_ZYx,
    RemapXYZAxisTo_ZyX,
    RemapXYZAxisTo_zXy,
    RemapXYZAxisTo_zxY,
    RemapXYZAxisTo_zYX,
    RemapXYZAxisTo_zyx,
    RemapXYZAxisTo_XYZ,
    MapPauliAxis;
