namespace Test {
    @EntryPoint()
    operation Test() : Unit {
        let foo = "hello!";
        Message(foo);
    }

    operation BadSyntax() {
    }
}
