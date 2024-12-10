namespace Kata.Verification {
    import Std.Convert.*;

    function IsSeven_Reference(x : Bool[]) : Bool {
        return BoolArrayAsInt(x) == 7;
    }

    @EntryPoint()
    function CheckSolution() : Bool {
        let N = 3;
        for k in 0..2^N - 1 {
            let x = IntAsBoolArray(k, N);

            let actual = Kata.IsSeven(x);
            let expected = IsSeven_Reference(x);

            if actual != expected {
                Message($"Failed on test case x = {x}: got {actual}, expected {expected}");
                return false;
            }
        }
        Message("Correct!");
        true
    }
}
