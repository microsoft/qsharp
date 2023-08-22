/// # Sample
/// Ternary
///
/// # Description
/// Q# supports conditional expressions (`if` expressions) without the use of a ternary operator.
namespace MyQuantumApp {
    @EntryPoint()
    operation Main() : Unit {
        let fahrenheit = 40;

        // `if` can also be used as an expression, to conditionally return a value.
        // This emulates the behavior of the ternary operator in languages like Python.
        let absoluteValue = if fahrenheit > 0 { fahrenheit } else { fahrenheit * -1 };        
    }
}