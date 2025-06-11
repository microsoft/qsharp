// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[derive(Copy, Clone, Debug)]
pub struct PauliNoise {
    /// Pauli noise distribution for sampling.
    /// When p is randomly distributed on [0.0, 1.0) the following error is applied:
    /// X, when p is from [0.0, distribution[0])
    /// Y, when p is from [distribution[0], distribution[1])
    /// Z, when p is from [distribution[1], distribution[2])
    /// I, when p is from [distribution[2], 1.0)
    pub distribution: [f64; 3],
}

impl Default for PauliNoise {
    fn default() -> Self {
        Self {
            distribution: [0.0; 3],
        }
    }
}

impl PauliNoise {
    pub fn from_probabilities(px: f64, py: f64, pz: f64) -> Result<Self, String> {
        let px_py = px + py;
        let px_py_pz = px_py + pz;
        if px < 0.0 || py < 0.0 || pz < 0.0 || px_py_pz > 1.0 {
            Err("Incorrect Pauli noise probabilities.".to_string())
        } else {
            Ok(Self {
                distribution: [px, px_py, px_py_pz],
            })
        }
    }

    #[must_use]
    pub fn is_noiseless(&self) -> bool {
        self.distribution[2] <= f64::EPSILON
    }
}
