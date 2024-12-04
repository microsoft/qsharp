namespace Kata.Verification {
    import Std.Convert.*;
    import Std.Random.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        mutable wins = 0;
        for i in 1..1000 {
            let x = DrawRandomBool(0.5);
            let y = DrawRandomBool(0.5);
            let (a, b) = (Kata.AliceClassical(x), Kata.BobClassical(y));
            if ((x and y) == (a != b)) {
                set wins = wins + 1;
            }
        }
        Message($"Win rate {IntAsDouble(wins) / 1000.}");
        if (wins < 700) {
            Message("Alice and Bob's classical strategy is not optimal");
            return false;
        }
        Message("Correct!");
        true
    }

}
