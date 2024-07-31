// # Sample
// Type Declarations
//
// # Description
// User-defined struct types are supported in Q#.
// They are immutable but support a copy-constructor syntax.

operation Main() : Unit {
    // Structs are defined with the `struct` keyword.
    struct Point3d { X : Double, Y : Double, Z : Double }

    // Structs can be instantiated with the `new` keyword followed
    // by the name of the struct type being initialized.
    let point = new Point3d { X = 1.0, Y = 2.0, Z = 3.0 };

    // Structs can be initialized by copying from another struct,
    // with modifications to specific fields specified.
    let point2 = new Point3d { ...point, Z = 4.0 };

    // Fields within a struct can be accessed by their name.
    // The below line accesses the field `x` on `point` by
    // name, using the item access operator `.`:
    let x : Double = point.X;
}
