// # Sample
// Copy and Update Operator
//
// # Description
// The copy and update operator in Q# is used to make a copy of an
// array and update a single element in the copied version.

function Main() : Unit {
    let array = [10, 11, 12, 13];

    // `w/` followed by the `<-` copies and updates a single element.

    // `new_array` is an array with values `[10, 11, 100, 13]`.
    // `array` is unchanged.
    let new_array = array w/ 2 <- 100;
    Message($"Updated array: {new_array}");

    // `new_array` is an array with values `[10, 100, 12, 200]`.
    // `array` is unchanged.
    let new_array = array
        w/ 1 <- 100
        w/ 3 <- 200;
    Message($"Updated array: {new_array}");

    // In addition to arrays, we can also copy-and-update structs.
    // First, let's define a struct called `Complex` which represents a
    // complex number.
    struct Complex { Real : Double, Imaginary : Double }
    // Instantiation of the above struct.
    let complex = new Complex { Real = 42.0, Imaginary = 0.0 };

    // `new_complex` is a new instance of the `Complex` struct with the
    // `Real` field updated to `100.0`.
    // `complex` is unchanged.
    let new_complex = new Complex { ...complex, Real = 100.0 };
}
