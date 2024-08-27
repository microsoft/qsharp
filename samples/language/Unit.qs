// # Sample
// Unit
//
// # Description
// The `Unit` type is the singleton type whose only value is ().
// Callables implicitly return `Unit` if no explicit or implicit
// return is specified.

function ExplicitReturn() : Unit {
    return ();
}

function NoReturn() : Unit {
    // No return, thus implicitly returning `Unit`.
}

function Main() : Unit {
    ExplicitReturn();
    NoReturn();
}