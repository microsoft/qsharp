/// # Sample
/// Double
///
/// # Description
/// A double-precision 64-bit floating-point number.
/// Values range from -1.79769313486232e308 to 1.79769313486232e308
/// as well as NaN (not a number).
/// Value literals for the Double type can be expressed in standard
/// or scientific notation.
namespace MyQuantumApp {

    @EntryPoint()
    operation Main() : Double {
        // A double declared in standard notation.
        let double = 0.1973269804;

        // A double declared in scientific notation.
        let double = 1.973269804e-1;

        return double;
    }
}