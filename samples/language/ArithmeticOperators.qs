/// # Sample
/// Arithmetic Operators
///
/// # Description
/// Arithmetic operators in Q# are used to perform basic mathematical operations
/// on numerical values. The arithmetic operators in Q# are `-` (negation), `+`,
/// `-`, `*`, `/`, `%`, and `^`.
namespace MyQuantumApp {

    @EntryPoint()
    operation Main() : Unit {

        // `-`, when applied to a single value on its right, negates the value.

        // The integer value `-32`.
        let integer = -32;

        // The integer value `32`.
        let integer = --32;

        // The integer value `-32`.
        let integer = ---32;

        // `+` adds two values together.

        // The integer value `42`.
        let integer = 10 + 32;

        // `-` subtracts the right-hand value from the left-hand value.

        // The integer value `-22`.
        let integer = 10 - 32;

        // `*` multiplies the two values together.

        // The integer value `320`.
        let integer = 10 * 32;

        // `/` divides the left-hand value by the right-hand value.
        // When the operands are both integers, the result is truncated to an integer.

        // The integer value `3`.
        let integer = 32 / 10;

        // The integer value `-3`.
        let integer = -32 / 10;

        // The double value `3.2`.
        let double = 32.0 / 10.0;

        // `%` gives the modulo of the left-hand value by the right-hand value.

        // The integer value `2`.
        let integer = 32 % 10;

        // The integer value `-2`.
        let integer = -32 % 10;

        // The integer value `2`.
        let integer = 32 % -10;

        // The integer value `-2`.
        let integer = -32 % -10;

        // `^` gives the exponent of the left-hand value raised to the power of the right-hand value.
        // When the operands are both integers, the right-hand value cannot be negative. This is
        // checked at runtime.

        // The integer value `81`.
        let integer = 3 ^ 4;

        // The integer value `-81`.
        let integer = -3 ^ 4;

        // The double value `256.0`.
        let double = 16.0 ^ 2.0;

        // The double value `-256.0`.
        let double = -16.0 ^ 2.0;

        // The double value `4.0`.
        let double = 16.0 ^ 0.5;

        // The double value `0.25`.
        let double = 16.0 ^ -0.5;
    }
}
