/// # Sample
/// Type Declarations
///
/// # Description
/// User-defined types, or UDTs as they are commonly called, are supported
/// in Q#. They are immutable but support a copy-and-update construct.
namespace MyQuantumApp {
    @EntryPoint()
    operation Main() : Unit {
        // UDTs are defined with the `newtype` keyword.
        newtype Point3d = (X: Double, Y: Double, Z: Double);

        // The items within a UDT can be either named or unnamed.
        newtype DoubleInt = (Double, ItemName: Int);

        // UDTs can also be nested.
        newtype Nested = (Double, (ItemName: Int, String));

        let point = Point3d(1.0, 2.0, 3.0);

        // Items within a UDT can be accessed either by their name,
        // or by deconstruction.
        // The below line accesses the field `x` on `point` by
        // name, using the item access operator `::`:
        let x: Double = point::X;

        // The below line accesses the field `x` via deconstruction:
        let (x, _, _) = point!;

        // The below line uses the unwrap operator `!` to access the entire
        // tuple. The type of `unwrappedTuple` is `(Double, Double, Double)`.
        let unwrappedTuple = point!;
    }
}