/// # Sample
/// Conditional Branching
///
/// # Description
/// Q# supports three branching keywords: `if`, `elif`, and `else`. These behave in the normal
/// way, if you're familiar with if expressions or statements in other languages. One key distinction
/// in Q# is that ifs are expressions, not statements. That means that they can return values, and
/// they effectively perform the job of both ternary expressions and if statements in other languages.
/// If expressions allow your code to branch, or conditionally execute parts of a Q# program.
namespace MyQuantumApp {
    @EntryPoint()
    operation Main() : Unit {
        let number = 5;
        // Conditionally messages "Fizz" if the `number`, in this case 5, is divisible by 3.
        // Since 5 is not divisible by 3, the message "Fizz" will not be printed.
        if number % 3 == 0 { Message("Fizz"); }

        // Conditionally messages "Buzz" if the `number`, in this case 5, is divisible by 5.
        // Since 5 is divisible by 5, the message "Buzz" will be printed.
        if number % 5 == 0 { Message("Buzz"); }

        let fahrenheit = 40;

        // In this example, we print a message based on the temperature. The
        // message that will be printed is "It is livable".
        // `elif` allows you to express successive conditional expressions, with each
        // successive condition only being evaluated if the previous one was false.
        if fahrenheit <= 32 {
            Message("It's freezing");
        } elif fahrenheit <= 85 {
            Message("It is livable");
        } else {
            Message("It is way too hot");
        }

        let fahrenheit = 40;

        // `if` can also be used as an expression, to conditionally return a value.
        let absoluteValue = if fahrenheit > 0 { fahrenheit } else { fahrenheit * -1 };        
    }
}