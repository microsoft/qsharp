// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
namespace Test {
    import Std.Math.*;
    import Std.Arrays.*;
    import Std.Convert.*;
    import Std.Random.*
    import Std.DurrHoyerLibrary.*;

    function MaxIntArray(arr : Int[]) : Int {
        mutable max = arr[0];
        for i in 1..(Length(arr) - 1) {
            if (arr[i] > max) {
                set max = arr[i];
            }
        }
        return max;
    }

    /// Shared logic for running the Durr-Hoyer test cases
    operation RunDurrHoyerTestWithShots(testLists : Int[][], expectedIndices : Int[], shots : Int, testType : String) : Unit {
        // Iterate over test cases
        for (list, expectedIndex) in Zipped(testLists, expectedIndices) {
            let maxValue = MaxIntArray(list);
            let double : Double = IntAsDouble(maxValue + 1);
            let log : Double = Log(double) / Log(2.0);
            let nQubits = Ceiling(log);
            let listSize = Length(list);


            // Variable to track how many times we find the correct index
            mutable correctCount = 0;

            // Run the Durr-Hoyer algorithm multiple times (shots)
            for _ in 1..shots {
                let candidate = DrawRandomInt(0, Length(list) - 1);
                let foundIndex : Int = DurrHoyerAlgorithm(list, nQubits, testType, candidate, listSize);

                // Check if the found index matches the expected index
                if (foundIndex == expectedIndex) {
                    set correctCount += 1;
                }
            }

            // Calculate the probability of finding the correct index
            let probability = IntAsDouble(correctCount) / IntAsDouble(shots);

            // Assert that the probability is above 50%
            Fact(probability >= 0.5, $"Probability of finding the {testType} for list {list} is less than 50%. Found: {probability * 100.0}%");

            // Optionally print debugging info
            Message($"List: {list}");
            Message($"Probability of finding the {testType} is {probability * 100.0}%");
        }
    }

    // Minimum test case using the shared logic
    operation RunDurrHoyerMinimumUnitTestWithShots(shots : Int) : Unit {
        let testLists = [
            [5, 3, 1, 2, 4],
            [6, 5, 4, 3, 1],
            [7, 5, 6, 1, 2]
        ];

        // Expected results (minimum element index for each list)
        let expectedMinIndices = [2, 4, 3];

        // Use the shared logic to run the test with "min" type
        RunDurrHoyerTestWithShots(testLists, expectedMinIndices, shots, "min");
    }

    // Maximum test case using the shared logic
    operation RunDurrHoyerMaximumUnitTestWithShots(shots : Int) : Unit {
        let testLists : Int[][] = [
            [2, 3, 1, 5, 4],
            [1, 5, 4, 3, 6],
            [7, 5, 6, 1, 2]
        ];

        // Expected results (maximum element index for each list)
        let expectedMaxIndices : Int[] = [3, 4, 0];

        // Use the shared logic to run the test with "max" type
        RunDurrHoyerTestWithShots(testLists, expectedMaxIndices, shots, "max");
    }
    operation RunDurrHoyerMaximumUnitTestWithShots(shots : Int) : Unit {
        let testLists : Int[][] = [
            [2, 3, 1, 5, 4],
            [1, 5, 4, 3, 6],
            [7, 5, 6, 1, 2]
        ];

        // Expected results (maximum element index for each list)
        let expectedMaxIndices : Int[] = [3, 4, 0];

        // Use the shared logic to run the test with "max" type
        RunDurrHoyerTestWithShots(testLists, expectedMaxIndices, shots, "max");
    }
    operation RunDurrHoyerZeroValuesUnitTestWithShots(shots : Int) : Unit {
    let testLists = [
        [0, 3, 1, 2, 4],
        [6, 0, 4, 3, 1],
        [7, 5, 6, 0, 2]
    ];

    // Expected results (minimum element index for each list)
    let expectedMinIndices = [0, 1, 3];

    // Use the shared logic to run the test with "min" type
    RunDurrHoyerTestWithShots(testLists, expectedMinIndices, shots, "min");
    }
}
