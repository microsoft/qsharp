// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Core {
    /// # Summary
    /// Returns the defined start value of the given range.
    ///
    /// # Input
    /// ## r
    /// Input range.
    ///
    /// # Output
    /// The defined start value of the given range.
    ///
    /// # Remarks
    /// A range expression's first element is `start`,
    /// its second element is `start+step`, third element is `start+step+step`, etc.,
    /// until `end` is passed.
    /// 
    /// Note that the defined start value of a range is the same as the first element of the sequence,
    /// unless the range specifies an empty sequence (for example, 2 .. 1).
    function RangeStart(r : Range) : Int {
        r::Start
    }
    
    
    /// # Summary
    /// Returns the defined end value of the given range,
    /// which is not necessarily the last element in the sequence.
    ///
    /// # Input
    /// ## r
    /// Input range.
    ///
    /// # Output
    /// The defined end value of the given range.
    ///
    /// # Remarks
    /// A range expression's first element is `start`,
    /// its second element is `start+step`, third element is `start+step+step`, etc.,
    /// until `end` is passed.
    /// 
    /// Note that the defined end value of a range can differ from the last element in the sequence specified by the range;
    /// for example, in a range 0 .. 2 .. 5 the last element is 4 but the end value is 5.
    function RangeEnd(r : Range) : Int {
        r::End
    }
    
    
    /// # Summary
    /// Returns the integer that specifies how the next value of a range is calculated.
    ///
    /// # Input
    /// ## r
    /// Input range.
    ///
    /// # Output
    /// The defined step value of the given range.
    ///
    /// # Remarks
    /// A range expression's first element is `start`,
    /// its second element is `start+step`, third element is `start+step+step`, etc.,
    /// until `end` is passed.
    function RangeStep(r : Range) : Int {
        r::Step
    }

    /// # Summary
    /// Returns a new range which is the reverse of the input range.
    ///
    /// # Input
    /// ## r
    /// Input range.
    ///
    /// # Output
    /// A new range that is the reverse of the given range.
    ///
    /// # Remarks
    /// Note that the reverse of a range is not simply `end`..`-step`..`start`, because
    /// the actual last element of a range may not be the same as `end`.
    function RangeReverse(r : Range) : Range {
        let start = r::Start + ((r::End - r::Start) / r::Step) * r::Step;
        start..-r::Step..r::Start
    }

    /// # Summary
    /// Returns true if and only if the input range is empty.
    ///
    /// # Input
    /// ## rng
    /// Any range
    ///
    /// # Output
    /// True, if and only if `rng` is empty
    ///
    /// # Remark
    /// This function needs to check at most one range index
    /// to determine whether the range is empty.
    function IsRangeEmpty(rng : Range) : Bool {
        for idx in rng {
            return false;
        }
        return true;
    }
}
