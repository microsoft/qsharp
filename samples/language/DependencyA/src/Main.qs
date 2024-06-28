namespace DependencyA {
    function MagicFunction() : Unit {
        Message("hello from dependency A!");
    }
    export MagicFunction, Microsoft.Quantum.Core.Length as Foo;
}
