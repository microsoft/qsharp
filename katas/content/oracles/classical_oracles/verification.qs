namespace Kata.Verification {

    open Microsoft.Quantum.Convert;

    // ------------------------------------------------------
    @EntryPoint()
    function CheckSolution(): Bool {
        let N = 3;
        for k in 0..((2^N)-1) {
            let x = IntAsBoolArray(k, N);

            let actual = Kata.IsSeven(x);
            let expected = IsSeven(x);

            if actual != expected {
                Message($"Failed on test case x = {x}: got {actual}, expected {expected}");
                return false;
            }
        }
        Message("All tests passed.");
        true
    }

}
