// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
namespace Test {
    import Std.Math.*;
    import Std.Arrays.*;
    import Std.Convert.*;
    import Std.Random.*
    import Std.DurrHoyerLibrary.*;

    // Function to find the maximum element in an array
    function MaxIntArray(arr : Int[]) : Int {
        mutable max = arr[0];
        for i in arr[1..Length(arr) - 1] {
            if (arr[i] > max) {
                set max = arr[i];
            }
        }
        return max;
    }

    // Common function to compute the probability of finding the minimum or maximum index
    operation ComputeDurrHoyerProbabilityWithShots(
        shots : Int,
        testLists : Int[][],
        expectedIndices : Int[],
        operationType : String
    ) : Unit {
        for (list, expectedIndex) in Zipped(testLists, expectedIndices) {
            let lengthList : Int = Length(list);
            let candidateIndex : Int = DrawRandomInt(0, lengthList - 1);
            let maxValue = MaxIntArray(list);
            let double : Double = IntAsDouble(maxValue + 1);
            let log : Double = Log(double) / Log(2.0);
            let nQubits = Ceiling(log);
            let testLists : Int[][] = [
                [5, 3, 1, 2, 4],
                [6, 5, 4, 3, 1],
                [7, 5, 6, 1, 2]
            ];
            // Variable to track how many times we find the correct index (min or max)
            mutable correctCount = 0;

            // Run the Durr-Hoyer algorithm multiple times (shots)
            for _ in 1..shots {
                let foundIndex : Int = DurrHoyerAlgorithm(list, nQubits, operationType, candidateIndex);

                // Check if the found index matches the expected index (min or max)
                if (foundIndex == expectedIndex) {
                    set correctCount += 1;
                }
            }

            // Calculate the probability of finding the correct index
            let probability = IntAsDouble(correctCount) / IntAsDouble(shots);

            // Assert that the probability is above 50%
            Assert(probability > 0.5, $"Probability of finding the {operationType} for list {list} is less than 50%. Found: {probability * 100.0}%");

            // Optionally print debugging info
            Message($"List: {list}");
            Message($"Probability of finding the {operationType} is {probability * 100.0}%");
        }
    }

    // Function to compute the probability of finding the minimum index using the common logic
    operation RunDurrHoyerMinimumUnitTestWithShots(testLists : Int[][], shots : Int) : Unit {

        let expectedMinIndices = [2, 4, 3];

        // Use the common logic for computing minimum
        ComputeDurrHoyerProbabilityWithShots(shots, testLists, expectedMinIndices, "min");
    }

    // Function to compute the probability of finding the maximum index using the common logic
    operation RunDurrHoyerMaximumUnitTestWithShots(testLists : Int[][], shots : Int) : Unit {

        let expectedMaxIndices = [0, 0, 0];

        // Use the common logic for computing maximum
        ComputeDurrHoyerProbabilityWithShots(shots, testLists, expectedMaxIndices, "max");
    }
}
