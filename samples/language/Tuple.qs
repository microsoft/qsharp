// # Sample
// Tuple
//
// # Description
// A tuple literal is a sequence of one or more expressions of any type,
// separated by commas and enclosed in parentheses `(` and `)`. The type of
// the tuple includes the information about each item type.
function Main() : (Int, String) {
    // A tuple of type `String`, `Int`, and `Double`
    let myTuple = ("Id", 0, 1.);
    Message($"Tuple: {myTuple}");

    // A tuple may be unpacked by assigning it to a tuple of variables
    let (name, n, d) = myTuple;
    Message($"Unpacked: {name}, {n}, {d}");

    // If not all items are needed, a tuple may be unpacked into
    // a tuple of variables with a wildcard `_` for the unwanted items.
    let (name, _, _) = myTuple;
    Message($"Name: {name}");

    // A single item in a round brackets does not denote a tuple. In this case,
    // the parentheses are used to group the expression, not to create a tuple.
    // Note that the type of `item` is `Int`, not `(Int)`.
    let item = (0);
    Message($"Item: {item}");

    // A tuple with a single item may be created with a trailing comma.
    // The type of `myTuple` is `(Int)` or, rather `(Int,)`, not `Int`.
    let myTuple = (0, );
    Message($"myTuple: {myTuple}");

    // A tuple of type `Pauli`, and a nested tuple of type `(Int, Int)`.
    // The type annotation is provided for clarity, but not necessary.
    let myTuple : (Pauli, (Int, Int)) = (PauliX, (3, 1));
    Message($"Tuple: {myTuple}");

    // A tuple containing a nested tuple may  be unpacked
    // into a tuple of variables with the same structure.
    let (p, (x, y)) = myTuple;
    Message($"Unpacked: {p}, {x}, {y}");

    // The outer tuple may be unpacked without unpacking the inner tuple.
    let (_, coords) = myTuple;
    Message($"Inner tuple: {coords}");

    return (0, "Foo");
}
