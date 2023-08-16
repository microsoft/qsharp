/// # Sample
/// Bool
///
/// # Description
/// The `Bool` type represents Boolean values. Possible values are `true`
/// or `false`.
namespace MyQuantumApp {

    @EntryPoint()
    operation Main() : Bool {
        // `Bool`s can be operated upon with boolean operators:
        let andOp = true and true;
        let orOp = false or true;

        // Comparisons return booleans:
        let eqComparison = 1 == 2;

        // `if` expressions use boolean expressions as their conditions:
        if (2 == 2) {
            // do something
        } else {
            // do something else
        }

        return true;
    }
}