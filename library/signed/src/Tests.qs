// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Std.Diagnostics.Fact;

/// This entrypoint runs tests for the signed integer library.
operation Main() : Unit {
    use a = Qubit[2];
    use b = Qubit[2];
    use c = Qubit[4];

    // 0b10 * 0b01 == 0b10 (1 * 2 = 2)
    X(a[0]);
    X(b[1]);
    TestOp(Operations.MultiplyI, a, b, c, 2);

    // 0b01 * 0b10 == 0b10 (1 * 2 = 2)
    X(a[1]);
    X(b[0]);
    TestOp(Operations.MultiplyI, a, b, c, 2);

    // 0b11 * 0b11 == 0b1001 (3 * 3 = 9)
    X(a[0]);
    X(b[0]);
    X(a[1]);
    X(b[1]);
    TestOp(Operations.MultiplyI, a, b, c, 9);
}

operation TestOp(op : (Qubit[], Qubit[], Qubit[]) => Unit, a : Qubit[], b : Qubit[], c : Qubit[], expect : Int) : Unit {
    op(a, b, c);
    let res = MeasureInteger(c);
    Fact(res == expect, $"Expected {expect}, got {res}");
    ResetAll(a + b + c);
}