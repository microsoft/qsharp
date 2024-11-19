// # Sample
// Class Constraints
//
// # Description
// Q# supports constraining generic types via _class constraints_. The formal term for this concept is bounded polymorphism,
// or parametric polymorphism.

// A generic type, or type parameter, is specified on a callable declaration to signify that a function can take multiple types of data as input.
// For a generic type parameter to be useful, we need to be able to know enough about it to operate on it. This is where class constraints come in. By specifying
// class constraints for a type parameter, we are limiting what types can be passed as arguments to a subset with known properties.

// Classes that Q# currently supports are:
// - `Eq`: denotes that a type can be compared to other values of the same type via the `==` operator.
// - `Add`: denotes that a type can be added to other values of the same type via the `+` operator.
// - `Mul`: denotes that a type can be used with the multiplication (`*`) operator.
// - `Mod`: denotes that a type can be used with the modulo (`%`) operator.
// - `Sub`: denotes that a type can subtracted from other values of the same type via the  `-` operator.
// - `Div`: denotes that a type can subtracted from other values of the same type via the  `/` operator.
// - `Signed`: denotes that a type can be negated or made positive with `+` or `-` unary operator prefixes.
// - `Ord`: denotes that a type can be used with comparison operators (`>`, `<`, `>=`, `<=`).
// - `Show`: denotes that a type can be converted to a string via format strings (`$"number: {num}"`).
// - `Exp['T]`: denotes that a type can be raised to a power of type `'T`. The return type of exponentiation is the type of the base.
// - `Integral`: denotes that a type is an integer-ish type, i.e., can be used in following expressions using the following operators: `&&&`, `|||`, `^^^`, `<<<`, and `>>>`.

// For example, we may want to write a function that checks if a list is full of entirely the same item. `f([3, 3, 3])` would be `true` and `f([3, 4])` would be false.
function AllEqual<'T : Eq>(items : 'T[]) : Bool {
    let allEqual = true;
    for i in 1..Length(items) - 1 {
        if items[i] != items[i - 1] {
            return false;
        }
    }
    return true;
}

function Main() : Unit {
    let is_equal = AllEqual([1, 1, 1]);
    Message($"{is_equal}");

    let is_equal = AllEqual([1, 2, 3]);
    Message($"{is_equal}");

    // Because we wrote this function generically, we are able to pass in different types, as
    // long as they can be compared via the class `Eq`.
    let is_equal = AllEqual([true, true, false]);
    Message($"{is_equal}");

    let is_equal = AllEqual(["a", "b"]);
    Message($"{is_equal}");

    let is_equal = AllEqual([[], [1]]);
    Message($"{is_equal}");

    let is_equal = AllEqual([[1], [1]]);
    Message($"{is_equal}");
}
