/// # Sample
/// Return Statement
///
/// # Description
/// Return statements are a form of control flow statement that abort the current callable and
/// return control flow to the callee's scope with a given value. 
namespace MyQuantumApp {
    @EntryPoint()
    operation Main() : Unit {
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

    function ReturnsUnit() : Unit {
        // If a callable returns `Unit`, no return statement is necessary.
        // If a callable is annotated to return any other type besides `Unit`, then it
        // must return a value of that type.
    }
}