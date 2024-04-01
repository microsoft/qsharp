// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub struct CatQubit {
    // physical error rate is computed as κ₁/κ₂
    pub(crate) k1_k2: f64,
}

impl Default for CatQubit {
    fn default() -> Self {
        // By default, we assume k1_k2 of 1e-5, arXiv:2302.06639 (p. 2).
        Self { k1_k2: 1e-5 }
    }
}

impl CatQubit {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
