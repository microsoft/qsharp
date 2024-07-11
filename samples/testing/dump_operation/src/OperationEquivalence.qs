namespace OperationEquivalence {
    open Microsoft.Quantum.Diagnostics;
    open SWAP;

    operation TestEquivalence(): Unit {
        let actual = qs => SWAP.ApplySWAP1(qs);
        let expected = qs => SWAP.ApplySWAP2(qs);
        Fact(CheckOperationsAreEqual(2, actual, expected)==true, "Actual and expected operation should be same");
    }
}
