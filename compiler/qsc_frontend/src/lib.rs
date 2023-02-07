// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::pedantic)]

mod lex;

#[must_use]
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
