namespace Kata {
    // 3 players, each player has own strategy and receives a bit from the referee
    operation PlayClassicalGHZ (strategies : (Bool => Bool)[], inputs : Bool[]) : Bool[] {
        let r = inputs[0];
        let s = inputs[1];
        let t = inputs[2];
        let a = strategies[0](r);
        let b = strategies[1](s);
        let c = strategies[2](t);
        return [a, b, c];
    }
}
