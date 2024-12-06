namespace Kata {
    import Std.Diagnostics.*;
    import Std.Arrays.*;

    @EntryPoint()
    operation OracleConverterDemo() : Unit {
        use register = Qubit[3];
        ApplyToEachA(H, register);

        Message("The equal superposition state:");
        DumpMachine();

        // Apply the phase oracle `IsSeven_PhaseOracle`
        IsSeven_PhaseOracle(register);

        Message("The state after applying the phase oracle IsSeven_PhaseOracle:");
        DumpMachine();
        ResetAll(register);

        ApplyToEachA(H, register);

        // Apply the marking oracle `IsSeven_MarkingOracle` as a phase oracle
        ApplyMarkingOracleAsPhaseOracle(IsSeven_MarkingOracle, register);

        Message("The state after applying the converted marking oracle IsSeven_MarkingOracle:");
        DumpMachine();
        ResetAll(register);
    }

    operation IsSeven_PhaseOracle(x : Qubit[]) : Unit is Adj + Ctl {
        Controlled Z(Most(x), Tail(x));
    }

    operation IsSeven_MarkingOracle(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        Controlled X(x, y);
    }

    operation ApplyMarkingOracleAsPhaseOracle(
        markingOracle : ((Qubit[], Qubit) => Unit is Adj + Ctl),
        qubits : Qubit[]
    ) : Unit is Adj + Ctl {
        use minus = Qubit();
        within {
            X(minus);
            H(minus);
        } apply {
            markingOracle(qubits, minus);
        }
    }
}
