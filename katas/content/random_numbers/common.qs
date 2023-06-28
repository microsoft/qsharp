    // ------------------------------------------------------
    /// # Summary
    /// Helper operation to rerun test operation several times
    /// (a single run can fail with non-negligible probability even for a correct solution).
    /// # Input
    /// ## testingHarness
    /// Test operation which verifies the user's solution.
    operation RetryTestOperation (testingHarness : (Unit => Bool)) : Unit {
        let numRetries = 3;
        mutable sufficientlyRandom = false;
        mutable attemptNum = 1;
        repeat {
            set sufficientlyRandom = testingHarness();
            set attemptNum += 1;
        } until (sufficientlyRandom or attemptNum > numRetries);

        if not sufficientlyRandom {
            fail $"Failed to generate sufficiently random integer";
        }
    }

    operation CheckRandomCalls () : Unit {
        Fact(GetOracleCallsCount(DrawRandomInt) == 0, "You are not allowed to call DrawRandomInt() in this task");
        Fact(GetOracleCallsCount(DrawRandomDouble) == 0, "You are not allowed to call DrawRandomDouble() in this task");
        ResetOracleCallsCount();
    }