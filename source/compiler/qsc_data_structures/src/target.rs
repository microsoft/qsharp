// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct TargetCapabilityFlags: u32 {
        const Adaptive = 0b0000_0001;
        const IntegerComputations = 0b0000_0010;
        const FloatingPointComputations = 0b0000_0100;
        const BackwardsBranching = 0b0000_1000;
        const HigherLevelConstructs = 0b0001_0000;
        const QubitReset = 0b0010_0000;
    }
}

impl std::str::FromStr for TargetCapabilityFlags {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "Base" => Ok(TargetCapabilityFlags::empty()),
            "Adaptive" => Ok(TargetCapabilityFlags::Adaptive),
            "IntegerComputations" => Ok(TargetCapabilityFlags::IntegerComputations),
            "FloatingPointComputations" => Ok(TargetCapabilityFlags::FloatingPointComputations),
            "BackwardsBranching" => Ok(TargetCapabilityFlags::BackwardsBranching),
            "HigherLevelConstructs" => Ok(TargetCapabilityFlags::HigherLevelConstructs),
            "QubitReset" => Ok(TargetCapabilityFlags::QubitReset),
            "Unrestricted" => Ok(TargetCapabilityFlags::all()),
            _ => Err(()),
        }
    }
}

impl Default for TargetCapabilityFlags {
    fn default() -> Self {
        TargetCapabilityFlags::empty()
    }
}
