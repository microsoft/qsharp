// # Sample
// BigInt
//
// # Description
// Value literals for the `BigInt` type are always postfixed with L and
// can be expressed in binary, octal, decimal, or hexadecimal representation.
// `BigInt`s can be arbitrarily large, as opposed to `Int`s which have a size limit.

function Main() : BigInt {
    // Numbers can be declared in hex, octal, decimal, or binary.
    let foo = 0x42L;
    Message($"Hexadecimal BigInt: {foo}");
    let foo = 0o42L;
    Message($"Octal BigInt: {foo}");
    let foo = 42L;
    Message($"Decimal BigInt: {foo}");
    let foo = 0b101010L;
    Message($"Binary BigInt: {foo}");

    // Numbers can be operated upon in the usual ways, with addition (+), subtraction (-),
    // multiplication (*), division (/), modulo (%), and exponentiation (^).
    let foo = foo + 1L;
    Message($"Addition result: {foo}");
    let foo = foo % 2L;
    Message($"Modulo result: {foo}");
    // `BigInt`s being raised to an exponent take an `Int` as the exponent.
    let foo = foo^2;
    Message($"Exponentiation result: {foo}");

    return foo;
}
