namespace Quantum.Kata.Reference {

    // ------------------------------------------------------
    @EntryPoint()
    function T11_IsSeven_ClassicalOracle () : Unit {
        let N = 3;
        for k in 0..((2^N)-1) {
            let x = IntAsBoolArray(k, N);

            let actual = IsSeven(x);
            let expected = IsSeven_Reference(x);

            Fact(actual == expected, $"    Failed on test case x = {x}: got {actual}, expected {expected}");
        }
    }

}
