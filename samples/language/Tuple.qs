/// # Sample
/// Tuple
///
/// # Description
/// A tuple literal is a sequence of one or more expressions of any type,
/// separated by commas and enclosed in parentheses `(` and `)`. The type of
/// the tuple includes the information about each item type.
/// 
/// Tuples containing a single item are treated as identical to the item
/// itself, both in type and value, which is called singleton tuple equivalence.
namespace MyQuantumApp {

    @EntryPoint()
    operation Main() : (Int, String) {
        // A tuple of type `String`, `Int`, and `Double`
        let myTuple: (String, Int, Double) = ("Id", 0, 1.);

        // A tuple of type `Pauli`, and a nested tuple of type `(Int, Int)`.
        let myTuple: (Pauli, (Int, Int)) = (PauliX, (3, 1));

        return (0, "Foo");
    }
}