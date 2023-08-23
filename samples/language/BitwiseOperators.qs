/// # Sample
/// Bitwise Operators
///
/// # Description
/// Bitwise operators in Q# perform operations on the bits of integer values,
/// producing a new integer value. The bitwise operators in Q# are
/// `~~~`, `&&&`, `|||`, `^^^`, `>>>`, and `<<<`.
namespace MyQuantumApp {

    @EntryPoint()
    operation Main() : Unit {

        // `~~~` performs a bitwise NOT on the bits of an integer.

        // The integer value -6.
        let integer = ~~~5;

        // The integer value 4.
        let integer = ~~~-5;

        // `&&&` performs a bitwise AND on the bits of two integer.

        // The integer value 4.
        let integer = 5 &&& 6;

        // The integer value 2.
        let integer = -5 &&& 6;

        // `|||` performs a bitwise OR on the bits of two integer.

        // The integer value 7.
        let integer = 5 ||| 6;

        // The integer value -1.
        let integer = -5 ||| 6;

        // `^^^` performs a bitwise XOR on the bits of two integer.

        // The integer value 3.
        let integer = 5 ^^^ 6;

        // The integer value -3.
        let integer = -5 ^^^ 6;

        // `>>>` performs a signed right bit-shift of a magnitude specified by the
        // right-hand integer on the bits of the left-hand integer.
        // If the right-hand integer is negative, it reverses the direction of the bit-shift.

        // The integer value 1.
        let integer = 5 >>> 2;

        // The integer value -2.
        let integer = -5 >>> 2;

        // The integer value 20.
        let integer = 5 >>> -2;

        // `<<<` performs a signed left bit-shift of a magnitude specified by the
        // right-hand integer on the bits of the left-hand integer.
        // If the right-hand integer is negative, it reverses the direction of the bit-shift.

        // The integer value 20.
        let integer = 5 <<< 2;

        // The integer value -20.
        let integer = -5 <<< 2;

        // The integer value 1.
        let integer = 5 <<< -2;
    }
}
