// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::SampledProfile;

use super::File;

#[test]
fn test_two_sampled() {
    let mut two_sampled = File::new(&["a", "b", "c", "d"]);

    let mut one = SampledProfile::new("one");
    one.push_sample(&[0, 1, 2], 1);
    one.push_sample(&[0, 1, 2], 1);
    one.push_sample(&[0, 1, 3], 4);
    one.push_sample(&[0, 1, 2], 3);
    one.push_sample(&[0, 1], 5);
    two_sampled.push(one);

    let mut two = SampledProfile::new("two");
    two.push_sample(&[0, 1, 2], 1);
    two.push_sample(&[0, 1, 2], 1);
    two.push_sample(&[0, 1, 3], 4);
    two.push_sample(&[0, 1, 2], 3);
    two.push_sample(&[0, 1], 5);
    two_sampled.push(two);

    println!("{}", serde_json::to_string(&two_sampled).unwrap());
}
