// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// # Summary
/// Maps a Pauli axis from one direction to another by applying an appropriate Clifford transformation.
/// For example, use `within MapPauliAxis(PauliZ, PauliX, q)` and perform Z rotation when X rotation is desired.
///
/// # Description
/// This function applies single-qubit Clifford transformations that remap Pauli axes on a Bloch sphere.
/// These mappings are useful to apply operations known for one Pauli basis in different Pauli bases.
/// Provide `from` and `to` parameters in terms of a passive transformation. For example,
/// when a rotation around the X axis is desired and a rotation around the Z axis is available,
/// the Z axis gets pointed in the direction of the X axis so that the rotation around Z axis can be used.
/// Out of several transformations that achieve the requested mapping, the one with fewer gates is used.
///
/// TODO: The table contains mix of active and passive language.
/// # Remarks
/// The complete list of possible mappings is provided for reference. \
/// +X+Y+Z ↦ +X+Y+Z: I (used when `from` == `to`)            \
/// +X+Y+Z ↦ +Z-Y+X: H (used when mapping Z to X and X to Z) \
/// +X+Y+Z ↦ +Y+Z+X: S⁻¹H (used when mapping Z to Y)         \
/// +X+Y+Z ↦ +Z+X+Y: HS (used when mapping Y to Z)           \
/// +X+Y+Z ↦ +Y-X+Z: S (used when mapping Y to X)            \
/// +X+Y+Z ↦ -Y+X+Z: S⁻¹ (used when mapping X to Y)          \
/// +X+Y+Z ↦ -X+Z+Y: S⁻¹HS                                   \
/// +X+Y+Z ↦ -Y-Z+X: SH                                      \
/// +X+Y+Z ↦ +Y-Z-X: SHZ, S⁻¹HY, YSH, SXH, S⁻¹YH, XS⁻¹H      \
/// +X+Y+Z ↦ -X-Z-Y: SHS⁻¹                                   \
/// +X+Y+Z ↦ +X-Y-Z: X                                       \
/// +X+Y+Z ↦ -Z+X-Y: HXS⁻¹, HSX, ZHS⁻¹, YHS, HYS, HS⁻¹Y      \
/// +X+Y+Z ↦ +X-Z+Y: SHS, HS⁻¹H                              \
/// +X+Y+Z ↦ -Y+Z-X: SHY, S⁻¹HZ, YS⁻¹H, SYH, S⁻¹XH, XSH      \
/// +X+Y+Z ↦ -X+Y-Z: Y                                       \
/// +X+Y+Z ↦ -Z+Y+X: HX, ZH                                  \
/// +X+Y+Z ↦ -Y-X-Z: YS, SX, S⁻¹Y, XS⁻¹                      \
/// +X+Y+Z ↦ +Y+X-Z: YS⁻¹, SY, S⁻¹X, XS                      \
/// +X+Y+Z ↦ +Z+Y-X: XH, HZ                                  \
/// +X+Y+Z ↦ +X+Z-Y: S⁻¹HS⁻¹, HSH                            \
/// +X+Y+Z ↦ -Z-X+Y: HXS, HSY, ZHS, YHS⁻¹, HYS⁻¹, HS⁻¹X      \
/// +X+Y+Z ↦ -X-Y+Z: Z                                       \
/// +X+Y+Z ↦ -Z-Y-X: YH, HY                                  \
/// +X+Y+Z ↦ +Z-X-Y: HS⁻¹                                    \
///
/// # Example
/// ```qsharp
/// // The following implements Rx(0.1, q) via Rz.
/// within {
///    MapPauliAxis(PauliZ, PauliX, q);
/// } apply {
///    Rz(0.1, q);
/// }
/// ```
///
/// # References
/// - [Wikipedia: Bloch sphere](https://wikipedia.org/wiki/Bloch_sphere)
/// - [Wikipedia: Clifford group](https://wikipedia.org/wiki/Clifford_group)
/// - [Wikipedia: Active and passive transformation](https://wikipedia.org/wiki/Active_and_passive_transformation)
operation MapPauliAxis(from : Pauli, to : Pauli, q : Qubit) : Unit is Adj + Ctl {
    if from == to {
        // X remains X, Y remains Y, and Z remains Z. No gates are applied.
    } elif (from == PauliZ and to == PauliX) or (from == PauliX and to == PauliZ) {
        // X becomes Z, Y becomes -Y, and Z becomes X. This is done by applying H gate.
        H(q);
    } elif from == PauliZ and to == PauliY {
        // X becomes Y, Y becomes Z, and Z becomes X. This is done by applying S⁻¹H gate sequence.
        Adjoint S(q);
        H(q);
    } elif from == PauliY and to == PauliZ {
        // X becomes Y, Y becomes Z, and Z becomes X. This is done by applying HS gate sequence.
        H(q);
        S(q);
    } elif from == PauliY and to == PauliX {
        // X becomes Y, Y becomes -X, and Z remains Z. This is done by applying S gate.
        S(q);
    } elif from == PauliX and to == PauliY {
        // X becomes -Y, Y becomes X, and Z remains Z. This is done by applying S⁻¹ gate.
        Adjoint S(q);
    } else {
        fail "Unsupported mapping of Pauli axes.";
    }
}

export
    MapPauliAxis;
