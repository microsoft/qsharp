/// # Sample
/// Int
///
/// # Description
/// A 64-bit signed integer.
/// Values range from -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807.
namespace MyQuantumApp {

    @EntryPoint()
    operation Main() : Int {
        // Numbers can be declared in hex, octal, decimal, or binary.
        let foo = 0x42;
        let foo = 0o42;
        let foo = 42;
        let foo = 0b101010;

        // Numbers can be operated upon in the usual ways, with addition (+), subtraction (-),
        // multiplication (*), division (/), modulo (%), and exponentiation (^).
        let foo = foo + 1;
        let foo = foo % 2;
        let foo = foo ^ 2;
        return foo;
    }
}