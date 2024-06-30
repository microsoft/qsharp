/// # Sample
/// Bitwise Operators
///
/// # Description
/// Bitwise operators in Q# perform operations on the bits of integer values,
/// producing a new integer value. The bitwise operators in Q# are
/// `~~~`, `&&&`, `|||`, `^^^`, `>>>`, and `<<<`.
namespace MyQuantumApp {

    @EntryPoint()
    function Main() : Unit {

        // `~~~` performs a bitwise NOT on the bits of an integer.

        // The integer value -6.
        let integer = ~~~5;
        Message($"Bitwise NOT: {integer}");

        // The integer value 4.
        let integer = ~~~-5;
        Message($"Bitwise NOT: {integer}");

        // `&&&` performs a bitwise AND on the bits of two integers.

        // The integer value 4.
        let integer = 5 &&& 6;
        Message($"Bitwise AND: {integer}");

        // The integer value 2.
        let integer = -5 &&& 6;
        Message($"Bitwise AND: {integer}");

        // `|||` performs a bitwise OR on the bits of two integers.

        // The integer value 7.
        let integer = 5 ||| 6;
        Message($"Bitwise OR: {integer}");

        // The integer value -1.
        let integer = -5 ||| 6;
        Message($"Bitwise OR: {integer}");

        // `^^^` performs a bitwise XOR on the bits of two integers.

        // The integer value 3.
        let integer = 5 ^^^ 6;
        Message($"Bitwise XOR: {integer}");

        // The integer value -3.
        let integer = -5 ^^^ 6;
        Message($"Bitwise XOR: {integer}");

        // `>>>` performs a signed right bit-shift of a magnitude specified by the
        // right-hand integer on the bits of the left-hand integer.
        // If the right-hand integer is negative, it reverses the direction of the bit-shift.

        // The integer value 1.
        let integer = 5 >>> 2;
        Message($"Right Bit-shift: {integer}");

        // The integer value -2.
        let integer = -5 >>> 2;
        Message($"Right Bit-shift: {integer}");

        // The integer value 20.
        let integer = 5 >>> -2;
        Message($"Right Bit-shift: {integer}");

        // `<<<` performs a signed left bit-shift of a magnitude specified by the
        // right-hand integer on the bits of the left-hand integer.
        // If the right-hand integer is negative, it reverses the direction of the bit-shift.

        // The integer value 20.
        let integer = 5 <<< 2;
        Message($"Left Bit-shift: {integer}");

        // The integer value -20.
        let integer = -5 <<< 2;
        Message($"Left Bit-shift: {integer}");

        // The integer value 1.
        let integer = 5 <<< -2;
        Message($"Left Bit-shift: {integer}");
    }
}
