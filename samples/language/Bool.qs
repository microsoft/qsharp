// # Sample
// Bool
//
// # Description
// The `Bool` type represents Boolean values. Possible values are `true`
// or `false`.

function Main() : Bool {
    // `Bool`s can be operated upon with boolean operators:
    let andOp = true and true;
    Message($"AND operation: {andOp}");

    let orOp = false or true;
    Message($"OR operation: {orOp}");

    // Comparisons return booleans:
    let eqComparison = 1 == 2;
    Message($"Equality comparison: {eqComparison}");

    // `if` expressions use boolean expressions as their conditions:
    if (2 == 2) {
        Message("2 equals 2");
        // do something
    } else {
        Message("2 does not equal 2");
        // do something else
    }

    return true;
}
