// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::cmp::max;

pub fn search_for_code_distances(
    num_rounds: usize,
    left_code_distance_indexes: &[usize],
    right_code_distance_indexes: &[usize],
    mut check_if_can_improve_with_increasing_code_distances: impl FnMut(&[usize]) -> bool,
) -> Option<Vec<usize>> {
    let mut right = right_code_distance_indexes.to_vec();

    if check_if_can_improve_with_increasing_code_distances(&right) {
        // If the output error rate is not sufficient even for Max code distances,
        // we should increase the number of rounds.
        return None;
    }

    if !check_if_can_improve_with_increasing_code_distances(left_code_distance_indexes) {
        // If the output error rate is sufficient the leftmost code distances,
        // we should skip the search.
        return None;
    }

    let mut mid = right.clone();

    for cur_round in 0..num_rounds {
        let mut left = mid.clone();
        left[cur_round] = if cur_round == 0 {
            left_code_distance_indexes[cur_round]
        } else {
            mid[cur_round - 1]
        };
        while left[cur_round] + 1 < right[cur_round] {
            mid[cur_round] = (left[cur_round] + right[cur_round]) / 2;
            if check_if_can_improve_with_increasing_code_distances(&mid) {
                left[cur_round] = mid[cur_round] + 1;
            } else {
                right[cur_round] = mid[cur_round];
            }
        }
        if left[cur_round] + 1 == right[cur_round] {
            mid[cur_round] = left[cur_round];
            if check_if_can_improve_with_increasing_code_distances(&mid) {
                mid[cur_round] = right[cur_round];
            }
        } else {
            mid[cur_round] = right[cur_round];
        }
    }

    Some(mid)
}

pub fn iterate_for_code_distances(
    num_rounds: usize,
    left_code_distance_indexes: &[usize],
    right_code_distance_indexes: &[usize],
    initial_code_distance_indexes: &[usize],
    mut check_if_can_improve_with_increasing_code_distances: impl FnMut(&[usize]) -> bool,
) {
    let mut current = initial_code_distance_indexes.to_vec();

    // validation: current >= left_code_distance_indexes
    for (index, item) in current.iter_mut().enumerate() {
        if *item < left_code_distance_indexes[index] {
            *item = left_code_distance_indexes[index];
        }
    }

    // we expect that the current was already checked before and there is not room for improvement.
    // here, we are switching to another branch of the search tree:
    // Example: 1, 2, 10 -> 1, 3, 3
    if !switch_to_non_comparable_code_distance_indexes(
        &mut current,
        left_code_distance_indexes,
        right_code_distance_indexes,
        num_rounds,
    ) {
        return;
    }

    loop {
        if check_if_can_improve_with_increasing_code_distances(&current) {
            if !increment_code_distance_indexes(
                &mut current,
                left_code_distance_indexes,
                right_code_distance_indexes,
                num_rounds,
            ) {
                return;
            }
        } else if !switch_to_non_comparable_code_distance_indexes(
            &mut current,
            left_code_distance_indexes,
            right_code_distance_indexes,
            num_rounds,
        ) {
            return;
        }
    }
}

fn increment_code_distance_indexes(
    current: &mut [usize],
    left_code_distance_indexes: &[usize],
    right_code_distance_indexes: &[usize],
    num_rounds: usize,
) -> bool {
    let mut i = num_rounds - 1;
    current[i] += 1;
    while current[i] > right_code_distance_indexes[i] {
        if i == 0 {
            // reached the leftmost index. cannot itrerate anymore.
            return false;
        }

        i -= 1;
        current[i] += 1;
    }

    while i < num_rounds - 1 {
        current[i + 1] = max(current[i], left_code_distance_indexes[i + 1]);
        i += 1;
    }

    true
}

fn switch_to_non_comparable_code_distance_indexes(
    current: &mut [usize],
    left_code_distance_indexes: &[usize],
    right_code_distance_indexes: &[usize],
    num_rounds: usize,
) -> bool {
    let mut j = current.len() - 1;
    while j > 0
        && (current[j] <= current[j - 1] + 1
            || current[j - 1] == right_code_distance_indexes[j - 1])
    {
        j -= 1;
    }

    if j == 0 {
        // reached the leftmost index. cannot itrerate anymore.
        return false;
    }

    current[j - 1] += 1;
    while j < num_rounds {
        current[j] = max(current[j - 1], left_code_distance_indexes[j]);
        j += 1;
    }

    true
}
