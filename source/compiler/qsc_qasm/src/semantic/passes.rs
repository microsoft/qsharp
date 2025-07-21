// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    semantic::{
        ast::{Block, BoxStmt, DelayStmt, GateCall},
        visit::Visitor,
    },
    stdlib::duration::Duration,
};

pub(crate) struct DurationAccumulator {
    durations: Vec<Duration>,
}

impl DurationAccumulator {
    fn new() -> Self {
        Self {
            durations: Vec::new(),
        }
    }

    /// Visits the block and accumulates the durations of all relevant
    /// statements that have a duration.
    /// Returns the total duration of all statements in the block.
    #[must_use]
    pub fn get_duration(scope: &Block) -> Duration {
        let mut accumulator = DurationAccumulator::new();
        accumulator.visit_block(scope);
        accumulator
            .durations
            .into_iter()
            .reduce(|acc, d| acc + d)
            .unwrap_or_default()
    }
}

impl Visitor for DurationAccumulator {
    fn visit_box_stmt(&mut self, stmt: &BoxStmt) {
        if let Some(duration) = &stmt.duration {
            if let Some(duration) = duration.get_const_duration() {
                self.durations.push(duration);
            }
        }
        super::visit::walk_box_stmt(self, stmt);
    }

    fn visit_gate_call_stmt(&mut self, stmt: &GateCall) {
        if let Some(duration) = &stmt.duration {
            if let Some(duration) = duration.get_const_duration() {
                self.durations.push(duration);
            }
        }
        super::visit::walk_gate_call_stmt(self, stmt);
    }

    fn visit_delay_stmt(&mut self, stmt: &DelayStmt) {
        if let Some(duration) = stmt.duration.get_const_duration() {
            self.durations.push(duration);
        }
        super::visit::walk_delay_stmt(self, stmt);
    }
}
