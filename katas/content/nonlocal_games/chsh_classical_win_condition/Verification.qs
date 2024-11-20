namespace Kata.Verification {
    import Std.Convert.*;

    function WinCondition_Reference(x : Bool, y : Bool, a : Bool, b : Bool) : Bool {
        return (x and y) == (a != b);
    }

    @EntryPoint()
    function CheckSolution() : Bool {
        for i in 0..1 <<< 4 - 1 {
            let bits = IntAsBoolArray(i, 4);
            let expected = WinCondition_Reference(bits[0], bits[1], bits[2], bits[3]);
            let actual = Kata.WinCondition(bits[0], bits[1], bits[2], bits[3]);

            if actual != expected {
                Message($"Win condition '{actual}' isn't as expected for X = {bits[0]}, Y = {bits[1]}, " +
                    $"A = {bits[2]}, B = {bits[3]}");
                return false;
            }
        }
        Message("Correct!");
        true
    }
}
