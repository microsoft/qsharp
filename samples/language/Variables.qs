// # Sample
// Variables
//
// # Description
// Variables in Q# are immutable by default and can be shadowed.

function Main() : Unit {
    // Immutable variables are declared with the `let` keyword:
    let immutableInt = 42;
    Message($"Immutable Int: {immutableInt}");

    // Mutable variables are declared with the `mutable` keyword:
    mutable mutableInt = 43;
    Message($"Mutable Int: {mutableInt}");

    // Mutable variables can be mutated with the `set` keyword:
    set mutableInt -= 1;
    Message($"Mutable Int after mutation: {mutableInt}");

    // This is not mutation, rather, this is declaring a new variable
    // entirely.
    let immutableInt = 43;
    let immutableInt = 0;
    Message($"Shadowed Immutable Int: {immutableInt}");

    // Structs can also be updated with a copy constructor and reassigned.
    struct Point3d { X : Double, Y : Double, Z : Double }

    mutable point = new Point3d { X = 0.0, Y = 0.0, Z = 0.0 };

    // The below line copies `point`, moves the X coordinate by +1.0,
    // and reassign the new `Point3d` to `point`.
    set point = new Point3d { ...point, X = point.X + 1.0 };

}

