/// # Sample
/// Ternary Expression
///
/// # Description
/// Q# supports ternary expressions with `?` and `|`.
/// The structure of a ternary is a boolean condition, followed by a `?`,
/// the expression to evaluate if the condition is `true`, a `|`, and the
/// expression to evaluate if the expression is `false`. More succinctly,
/// `[condition] ? [true branch] | [else branch]`
namespace MyQuantumApp {
    @EntryPoint()
    operation Main() : Unit {
        let fahrenheit = 40;

        // The below ternary expression sets the value of `fahrenheit` to its absolute value.
        // `fahrenheit` if `fahrenheit` is positive,
        // `fahrenheit * -1` if `fahrenheit` is negative.
        let absoluteValue = fahrenheit > 0 ? fahrenheit | fahrenheit * -1;
    }
}