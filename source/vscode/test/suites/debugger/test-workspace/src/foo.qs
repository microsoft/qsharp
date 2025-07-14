namespace Foo {
    open Bar;
    @EntryPoint()
    operation Foo() : Unit {
        Bar();
        Bar();
        use q = Qubit();
        H(q);
        Reset(q);
        let a = 2;
        AnotherCallFrame();
    }

    function AnotherCallFrame() : Unit {
        let b = 3;
        let c = 4;
        let d = 5;
    }
}
