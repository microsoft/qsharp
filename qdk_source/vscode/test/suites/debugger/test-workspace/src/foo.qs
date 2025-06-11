namespace Foo {
    open Bar;
    @EntryPoint()
    operation Foo() : Unit {
        Bar();
        Bar();
        use q = Qubit();
        H(q);
        Reset(q);
    }
}
