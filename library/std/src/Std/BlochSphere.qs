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
/// # Remarks
/// The complete list of possible mappings and a list of gate sequences to achieve them.
/// For example, a transformation of +X+Y+Z ↦ +X+Z-Y means that the bloch sphere is rotated so that
/// the X axis remains unchanged, Y axis points in Z direction, and Z axis points in -Y direction.
/// (Y with direction reversed). Such transformation could be used to achieve PauliZ to PauliY mapping.
///
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
/// +X+Y+Z ↦ +Z-X-Y: HS⁻¹
///
/// # Input
/// ## from
/// The Pauli axis to map from. Perform subsequent operations on this axis.
/// ## to
/// The Pauli axis to map to. Subsequent operations on `from` axis will perform as if they act on this axis.
/// ## q
/// The qubit on which the transformation will be applied.
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
        // +X+Y+Z ↦ +X+Y+Z, No gates are needed.
    } elif (from == PauliZ and to == PauliX) or (from == PauliX and to == PauliZ) {
        // +X+Y+Z ↦ +Z-Y+X
        H(q);
    } elif from == PauliZ and to == PauliY {
        // +X+Y+Z ↦ +Y+Z+X
        Adjoint S(q);
        H(q);
    } elif from == PauliY and to == PauliZ {
        // +X+Y+Z ↦ +Z+X+Y
        H(q);
        S(q);
    } elif from == PauliY and to == PauliX {
        // +X+Y+Z ↦ +Y-X+Z
        S(q);
    } elif from == PauliX and to == PauliY {
        // +X+Y+Z ↦ -Y+X+Z
        Adjoint S(q);
    } else {
        fail "Unsupported mapping of Pauli axes.";
    }
}

export
    MapPauliAxis;
