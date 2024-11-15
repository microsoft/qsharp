namespace Kata.Verification {
    import Std.Diagnostics.*;
    import KatasUtils.*;
    import Std.Math.*;
    import Std.Random.*;

    operation ShorEncode(qs : Qubit[]) : Unit is Adj + Ctl {
        BitflipEncode(qs[0..3..8]);
        ApplyToEachCA(H, qs[0..3..8]);
        for i in 0..2 {
            BitflipEncode(qs[3 * i..3 * i + 2]);
        }
    }

    operation BitflipEncode(qs : Qubit[]) : Unit is Adj + Ctl {
        CNOT(qs[0], qs[1]);
        CNOT(qs[0], qs[2]);
    }


    @EntryPoint()
    operation CheckSolution() : Bool {
        for err_ind in -1..8 {
            for err in [PauliX, PauliZ, PauliY] {
                for _ in 1..10 {
                    mutable correct = true;
                    mutable msg = "";
                    use qs = Qubit[9];
                    let theta = DrawRandomDouble(0.0, 1.0);
                    within {
                        // Prepare logical state on first qubit
                        Ry(2.0 * theta * PI(), qs[0]);
                        // Encode the state in multiple qubits
                        ShorEncode(qs);
                        // Introduce the error
                        if err_ind > -1 {
                            if err == PauliX {
                                X(qs[err_ind]);
                            } elif err == PauliZ {
                                Z(qs[err_ind]);
                            } else {
                                Y(qs[err_ind]);
                            }
                        }
                    } apply {
                        // Call solution to detect error
                        let (detected_ind, detected_err) = Kata.ShorDetectError(qs);
                        // Check that it is correct
                        if err_ind == -1 {
                            // No error
                            if detected_ind != -1 {
                                set correct = false;
                                set msg = $"There was no error, but the solution detected error at qubit {detected_ind}";
                            }
                        } else {
                            // There was an error
                            if detected_err != err {
                                set correct = false;
                                set msg = $"There was a {err} error, but the solution detected {detected_err} error";
                            } else {
                                if err == PauliX or err == PauliY {
                                    if detected_ind != err_ind {
                                        set correct = false;
                                        set msg = $"There was a {err} error at qubit {err_ind}, but the solution detected it at qubit {detected_ind}";
                                    }
                                } else {
                                    // For PauliZ errors, cannot say for certain in which qubit of the triplet it happened, so identify triplet
                                    if detected_ind != err_ind / 3 {
                                        set correct = false;
                                        set msg = $"There was a {err} error at qubit {err_ind}, but the solution detected it at qubit triplet {detected_ind}";
                                    }
                                }
                            }
                        }
                    }
                    // Check that the state was not modified by the solution
                    if not CheckAllZero(qs) {
                        set correct = false;
                        set msg = "The state of the qubits changed after the solution was applied";
                    }

                    if not correct {
                        Message("Incorrect.");
                        Message(msg);
                        ResetAll(qs);
                        return false;
                    }
                }
            }
        }

        Message("Correct!");
        true
    }
}
