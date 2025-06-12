// # Sample
// While Loops
//
// # Description
// While loops are a basic looping looping structure in Q# for repeatedly
// executing the contained code block for as long as the controlling
// condition evaluates to `true`.
// They can be used in both `operation` and `function` callables.

function Main() : Unit {
    mutable x = 0;
    while x < 3 {
        x += 1;
    }
}
