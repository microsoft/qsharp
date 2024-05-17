// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::str::FromStr;

use qsc_data_structures::target::TargetCapabilityFlags;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Profile {
    Unrestricted,
    Base,
    AdaptiveRI,
}

impl Profile {
    #[must_use]
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Unrestricted => "Unrestricted",
            Self::Base => "Base",
            Self::AdaptiveRI => "Adaptive_RI",
        }
    }
}

impl From<Profile> for TargetCapabilityFlags {
    fn from(value: Profile) -> Self {
        match value {
            Profile::Unrestricted => Self::all(),
            Profile::Base => Self::empty(),
            Profile::AdaptiveRI => Self::Adaptive | Self::QubitReset | Self::IntegerComputations,
        }
    }
}

impl FromStr for Profile {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Adaptive_RI" | "adaptive_ri" => Ok(Self::AdaptiveRI),
            "Base" | "base" => Ok(Self::Base),
            "Unrestricted" | "unrestricted" => Ok(Self::Unrestricted),
            _ => Err(()),
        }
    }
}
