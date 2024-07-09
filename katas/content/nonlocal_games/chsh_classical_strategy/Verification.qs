namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Random;

    @EntryPoint()
    operation CheckSolution() : Bool {
        mutable wins = 0;
        for i in 1..1000 {
            let x = DrawRandomInt(0, 1) == 1 ? true | false;
            let y = DrawRandomInt(0, 1) == 1 ? true | false;
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
        else {
            Message("Correct!");
        }
        true
    }

}
