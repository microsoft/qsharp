/// # Sample
/// Variables
///
/// # Description
/// Variables in Q# are immutable by default and can be shadowed.
namespace MyQuantumApp {
    @EntryPoint()
    operation Main() : Unit {
        // Immutable variables are declared with the `let` keyword:
        let immutableInt = 42;

        // Mutable variables are declared with the `mutable` keyword:
        mutable mutableInt = 43;

        // Mutable variables can be mutated with the `set` keyword:
        set mutableInt -= 1;

        // All variables can be shadowed by symbols introduced later in scope.
        // This is not mutation, rather, this is declaring a new variable
        // entirely.
        let immutableInt = 43;
        let immutableInt = 0;

        // UDTs can also be updated with copy-and-update expressions (`w/`)
        // or evaluate-and-reassign expressions (`w/=`).
        newtype Point3d = (X: Double, Y: Double, Z: Double);

        mutable point = Point3d(0.0, 0.0, 0.0);

        // The below line mutates `point`, moving the X coordinate by +1.0,
        // using evaluate-and-reassign.
        set point w/= X <- point::X + 1.0;

        // The below line also mutates `point`, moving the X coordinate by -1.0,
        // using copy-and-update.
        set point = point w/ X <- point::X + 1.0;
         
    }

}