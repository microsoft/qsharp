namespace Foo {
    open Bar;
    @Test()
    operation Foo() : Unit {
        Bar();
        Bar();
        use q = Qubit();
        H(q);
        Reset(q);
    }
}
