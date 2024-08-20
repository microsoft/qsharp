/// # Sample
/// Int
///
/// # Description
/// A 64-bit signed integer.
/// Values range from -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807.
namespace MyQuantumApp {

    function otherFunc() : Unit {

    }
    @EntryPoint()
    function Main() : Int {
    
        // Numbers can be declared in hex, octal, decimal, or binary.
        let foo = 0x42;
        Message($"Hexadecimal: {foo}");

        let foo = 0o42;
        Message($"Octal: {foo}");

        let foo = 42;
        Message($"Decimal: {foo}");

        let foo = 0b101010;
        Message($"Binary: {foo}");

        // Numbers can be operated upon in the usual ways, with addition (+), subtraction (-),
        // multiplication (*), division (/), modulo (%), and exponentiation (^).
        let foo = foo + 1;
        Message($"After addition: {foo}");

        let foo = foo % 2;
        Message($"After modulo: {foo}");

        let foo = foo^2;
        Message($"After exponentiation: {foo}");

        return foo;
    }
}
