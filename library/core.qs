// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Core {

    /// # Summary
    /// Returns the number of elements in an array.
    ///
    /// # Input
    /// ## a
    /// Input array.
    ///
    /// # Output
    /// The total count of elements in an array.
    ///
    function Length<'T>(a : 'T[]) : Int {
        a::Length
    }

    /// # Summary
    /// Returns the defined start value of the given range.
    ///
    /// # Input
    /// ## range
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
    function RangeStart(range : Range) : Int {
        range::Start
    }
    
    
    /// # Summary
    /// Returns the defined end value of the given range,
    /// which is not necessarily the last element in the sequence.
    ///
    /// # Input
    /// ## range
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
    function RangeEnd(range : Range) : Int {
        range::End
    }
    
    
    /// # Summary
    /// Returns the integer that specifies how the next value of a range is calculated.
    ///
    /// # Input
    /// ## range
    /// Input range.
    ///
    /// # Output
    /// The defined step value of the given range.
    ///
    /// # Remarks
    /// A range expression's first element is `start`,
    /// its second element is `start+step`, third element is `start+step+step`, etc.,
    /// until `end` is passed.
    function RangeStep(range : Range) : Int {
        range::Step
    }

    /// # Summary
    /// Returns a new range which is the reverse of the input range.
    ///
    /// # Input
    /// ## range
    /// Input range.
    ///
    /// # Output
    /// A new range that is the reverse of the given range.
    ///
    /// # Remarks
    /// Note that the reverse of a range is not simply `end`..`-step`..`start`, because
    /// the actual last element of a range may not be the same as `end`.
    function RangeReverse(range : Range) : Range {
        let start = range::Start + ((range::End - range::Start) / range::Step) * range::Step;
        start..-range::Step..range::Start
    }

    function AsString<'T>(v : 'T) : String {
        body intrinsic;
    }

}
