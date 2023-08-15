/// # Sample
/// BigInt
///
/// # Description
/// Value literals for the `BigInt` type are always postfixed with L and
/// can be expressed in binary, octal, decimal, or hexadecimal representation.
/// `BigInt`s can be arbitrarily large, as opposed to `Int`s which have a size limit.
namespace MyQuantumApp {

    @EntryPoint()
    operation Main() : BigInt {
        // Numbers can be declared in hex, octal, decimal, or binary.
        let foo = 0x42L;
        let foo = 0o42L;
        let foo = 42L;
        let foo = 0b101010L;

        // Numbers can be operated upon in the usual ways, with addition (+), subtraction (-),
        // multiplication (*), division (/), modulo (%), and exponentiation (^).
        let foo = foo + 1L;
        let foo = foo % 2L;
        // `BigInt`s being raised to an exponent take an `Int` as the exponent.
        let foo = foo ^ 2;
        return foo;
    }
}
