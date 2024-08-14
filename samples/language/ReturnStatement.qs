/// # Sample
/// Return Statement
///
/// # Description
/// Return statements are a form of control flow statement that abort the current callable and
/// return control flow to the callee's scope with a given value.
namespace MyQuantumApp {
    @EntryPoint()
    function Main() : Unit {
        // After the execution of the function `Returns42`, control flow
        // will return to this scope.
        let number42 = Returns42();
        // No return is required here, because the `Main` callable returns `Unit`.
    }

    function Returns42() : Int {
        // The type of the expression passed to a return statement must
        // line up with the return type annotation on the callable signature.
        // Here, the expression is a literal `Int` `42`.
        return 42;
        // Any code following a return statement is inaccessible.
    }

    function Returns42Implicit() : Int {
        // If the last expression is not followed by a semicolon,
        // its value is implicitly returned, in which case the return keyword is not necessary.
        42 // This literal value is implicitly returned.
        // Any code following an implicit return would cause the compilation to fail.
    }

    function ReturnsUnit() : Unit {
        // If a callable returns `Unit`, no return statement is necessary.
        // If a callable is annotated to return any other type besides `Unit`, then it
        // must return a value of that type.
    }
}
