namespace Kata.Verification {
    import Std.Convert.*;

    function WinCondition_Reference(rst : Bool[], abc : Bool[]) : Bool {
        return (rst[0] or rst[1] or rst[2]) == (abc[0] != abc[1] != abc[2]);
    }

    // All possible starting bits (r, s and t) that the referee can give
    // to Alice, Bob and Charlie.
    function RefereeBits() : Bool[][] {
        return [
            [false, false, false],
            [true, true, false],
            [false, true, true],
            [true, false, true]
        ];
    }

    @EntryPoint()
    function CheckSolution() : Bool {
        for rst in RefereeBits() {
            for i in 0..1 <<< 3 - 1 {
                let abc = IntAsBoolArray(i, 3);
                let expected = WinCondition_Reference(rst, abc);
                let actual = Kata.WinCondition(rst, abc);

                if actual != expected {
                    Message($"Win condition '{actual}' is wrong for rst={rst}, abc={abc}");
                    return false;
                }
            }
        }
        Message("Correct!");
        true
    }
}
