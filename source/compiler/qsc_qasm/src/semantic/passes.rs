// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    semantic::{
        ast::{BoxStmt, DelayStmt, GateCall},
        visit::Visitor,
    },
    stdlib::duration::Duration,
};

pub(crate) struct DurationAccumulator {
    pub(crate) durations: Vec<Duration>,
}

impl DurationAccumulator {
    pub fn new() -> Self {
        Self {
            durations: Vec::new(),
        }
    }

    pub fn get_duration(&self) -> Duration {
        self.durations
            .iter()
            .copied()
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
