// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use rustc_hash::{FxHashMap, FxHashSet};
use serde::Serialize;
use std::{cmp, fmt::Display, fmt::Write, ops::Not, vec};

/// Representation of a quantum circuit.
/// Implementation of `CircuitData` type from `qsharp-lang` npm package.
#[derive(Clone, Serialize, Default, Debug, PartialEq)]
pub struct Circuit {
    pub operations: Vec<Operation>,
    pub qubits: Vec<Qubit>,
}

#[derive(Clone, Serialize, Debug, PartialEq)]
pub struct Operation {
    #[allow(clippy::struct_field_names)]
    pub gate: String,
    #[serde(rename = "displayArgs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_args: Option<String>,
    #[serde(rename = "isControlled")]
    #[serde(skip_serializing_if = "Not::not")]
    pub is_controlled: bool,
    #[serde(rename = "isAdjoint")]
    #[serde(skip_serializing_if = "Not::not")]
    pub is_adjoint: bool,
    #[serde(rename = "isMeasurement")]
    #[serde(skip_serializing_if = "Not::not")]
    pub is_measurement: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<Register>,
    pub targets: Vec<Register>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Operation>,
}

const QUANTUM_REGISTER: usize = 0;
const CLASSICAL_REGISTER: usize = 1;

#[derive(Serialize, Debug, Eq, Hash, PartialEq, Clone)]
pub struct Register {
    #[serde(rename = "qId")]
    pub q_id: usize,
    pub r#type: usize,
    #[serde(rename = "cId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c_id: Option<usize>,
}

impl Register {
    pub fn quantum(q_id: usize) -> Self {
        Self {
            q_id,
            r#type: QUANTUM_REGISTER,
            c_id: None,
        }
    }

    pub fn classical(q_id: usize, c_id: usize) -> Self {
        Self {
            q_id,
            r#type: CLASSICAL_REGISTER,
            c_id: Some(c_id),
        }
    }
}

#[derive(PartialEq, Clone, Serialize, Debug)]
pub struct Qubit {
    pub id: usize,
    #[serde(rename = "numChildren")]
    pub num_children: usize,
}

#[derive(Clone, Debug, Copy, Default)]
pub struct Config {
    /// Perform Base Profile decompositions
    pub base_profile: bool,
    /// Maximum number of operations the builder will add to the circuit
    pub max_operations: usize,
}

impl Config {
    /// Set to the current UI limit + 1 so that it still triggers
    /// the "this circuit has too many gates" warning in the UI.
    /// (see npm\qsharp\ux\circuit.tsx)
    ///
    /// A more refined way to do this might be to communicate the
    /// "limit exceeded" state up to the UI somehow.
    pub const DEFAULT_MAX_OPERATIONS: usize = 10001;
}

type ObjectsByColumn = FxHashMap<usize, CircuitObject>;

struct Row {
    wire: Wire,
    objects: ObjectsByColumn,
    next_column: usize,
}

enum Wire {
    Qubit { q_id: usize },
    Classical { start_column: Option<usize> },
}

enum CircuitObject {
    Blank,
    Wire,
    WireCross,
    WireStart,
    DashedCross,
    Vertical,
    VerticalDashed,
    Object(String),
}

impl Row {
    fn add_object(&mut self, column: usize, object: &str) {
        self.add(column, CircuitObject::Object(object.to_string()));
    }

    fn add_gate(&mut self, column: usize, gate: &str, args: Option<&str>, is_adjoint: bool) {
        let mut gate_label = String::new();
        gate_label.push_str(gate);
        if is_adjoint {
            gate_label.push('\'');
        }
        if let Some(args) = args {
            let _ = write!(&mut gate_label, "({args})");
        }

        self.add_object(column, gate_label.as_str());
    }

    fn add_vertical(&mut self, column: usize) {
        if !self.objects.contains_key(&column) {
            match self.wire {
                Wire::Qubit { .. } => self.add(column, CircuitObject::WireCross),
                Wire::Classical { start_column } => {
                    if start_column.is_some() {
                        self.add(column, CircuitObject::WireCross);
                    } else {
                        self.add(column, CircuitObject::Vertical);
                    }
                }
            }
        }
    }

    fn add_dashed_vertical(&mut self, column: usize) {
        if !self.objects.contains_key(&column) {
            match self.wire {
                Wire::Qubit { .. } => self.add(column, CircuitObject::DashedCross),
                Wire::Classical { start_column } => {
                    if start_column.is_some() {
                        self.add(column, CircuitObject::DashedCross);
                    } else {
                        self.add(column, CircuitObject::VerticalDashed);
                    }
                }
            }
        }
    }

    fn start_classical(&mut self, column: usize) {
        self.add(column, CircuitObject::WireStart);
        if let Wire::Classical { start_column } = &mut self.wire {
            start_column.replace(column);
        }
    }

    fn add(&mut self, column: usize, circuit_object: CircuitObject) {
        self.objects.insert(column, circuit_object);
        self.next_column = column + 1;
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, columns: &[Column]) -> std::fmt::Result {
        // Temporary string so we can trim whitespace at the end
        let mut s = String::new();
        match &self.wire {
            Wire::Qubit { q_id: label } => {
                s.write_str(&fmt_qubit_label(*label))?;
                for (column_index, column) in columns.iter().enumerate().skip(1) {
                    let val = self.objects.get(&column_index);
                    let object = val.unwrap_or(&CircuitObject::Wire);

                    s.write_str(&column.fmt_qubit_circuit_object(object))?;
                }
            }
            Wire::Classical { start_column } => {
                for (column_index, column) in columns.iter().enumerate() {
                    let val = self.objects.get(&column_index);

                    let object = match (val, start_column) {
                        (Some(v), _) => v,
                        (None, Some(s)) if column_index > *s => &CircuitObject::Wire,
                        _ => &CircuitObject::Blank,
                    };

                    s.write_str(&column.fmt_classical_circuit_object(object))?;
                }
            }
        }
        writeln!(f, "{}", s.trim_end())?;
        Ok(())
    }
}

const MIN_COLUMN_WIDTH: usize = 7;

const QUBIT_WIRE: [char; 3] = ['─', '─', '─']; // "───────"
const CLASSICAL_WIRE: [char; 3] = ['═', '═', '═']; // "═══════"
const QUBIT_WIRE_CROSS: [char; 3] = ['─', '┼', '─']; // "───┼───"
const CLASSICAL_WIRE_CROSS: [char; 3] = ['═', '╪', '═']; // "═══╪═══"
const CLASSICAL_WIRE_START: [char; 3] = [' ', '╘', '═']; // "   ╘═══"
const QUBIT_WIRE_DASHED_CROSS: [char; 3] = ['─', '┆', '─']; // "───┆───"
const CLASSICAL_WIRE_DASHED_CROSS: [char; 3] = ['═', '┆', '═']; // "═══┆═══"
const VERTICAL_DASHED: [char; 3] = [' ', '┆', ' ']; // "   │   "
const VERTICAL: [char; 3] = [' ', '│', ' ']; // "   ┆   "
const BLANK: [char; 3] = [' ', ' ', ' ']; // "       "

/// "q_0  "
#[allow(clippy::doc_markdown)]
fn fmt_qubit_label(id: usize) -> String {
    let rest = MIN_COLUMN_WIDTH - 2;
    format!("q_{id: <rest$}")
}

struct Column {
    column_width: usize,
}

impl Default for Column {
    fn default() -> Self {
        Self {
            column_width: MIN_COLUMN_WIDTH,
        }
    }
}

impl Column {
    fn new(column_width: usize) -> Self {
        // Column widths should be odd numbers for this struct to work well
        let odd_column_width = column_width | 1;
        Self {
            column_width: odd_column_width,
        }
    }

    /// "── A ──"
    fn fmt_on_qubit_wire(&self, obj: &str) -> String {
        let column_width = self.column_width;
        format!("{:─^column_width$}", format!(" {obj} "))
    }

    /// "══ A ══"
    fn fmt_on_classical_wire(&self, obj: &str) -> String {
        let column_width = self.column_width;
        format!("{:═^column_width$}", format!(" {obj} "))
    }

    fn expand_template(&self, template: &[char; 3]) -> String {
        let half_width = self.column_width / 2;
        let left = template[0].to_string().repeat(half_width);
        let right = template[2].to_string().repeat(half_width);

        format!("{left}{}{right}", template[1])
    }

    fn fmt_classical_circuit_object(&self, circuit_object: &CircuitObject) -> String {
        if let CircuitObject::Object(label) = circuit_object {
            return self.fmt_on_classical_wire(label.as_str());
        }

        let template = match circuit_object {
            CircuitObject::Blank => BLANK,
            CircuitObject::Wire => CLASSICAL_WIRE,
            CircuitObject::WireCross => CLASSICAL_WIRE_CROSS,
            CircuitObject::WireStart => CLASSICAL_WIRE_START,
            CircuitObject::DashedCross => CLASSICAL_WIRE_DASHED_CROSS,
            CircuitObject::Vertical => VERTICAL,
            CircuitObject::VerticalDashed => VERTICAL_DASHED,
            CircuitObject::Object(_) => unreachable!("This case is covered in the early return."),
        };

        self.expand_template(&template)
    }

    fn fmt_qubit_circuit_object(&self, circuit_object: &CircuitObject) -> String {
        if let CircuitObject::Object(label) = circuit_object {
            return self.fmt_on_qubit_wire(label.as_str());
        }

        let template = match circuit_object {
            CircuitObject::Blank => BLANK,
            CircuitObject::Wire => QUBIT_WIRE,
            CircuitObject::WireCross => QUBIT_WIRE_CROSS,
            CircuitObject::WireStart => BLANK, // This should never happen
            CircuitObject::DashedCross => QUBIT_WIRE_DASHED_CROSS,
            CircuitObject::Vertical => VERTICAL,
            CircuitObject::VerticalDashed => VERTICAL_DASHED,
            CircuitObject::Object(_) => unreachable!("This case is covered in the early return."),
        };

        self.expand_template(&template)
    }
}

impl Display for Circuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rows = vec![];

        // Maintain a mapping from from Registers in the Circuit schema
        // to row in the diagram
        let mut register_to_row = FxHashMap::default();

        // Keep track of which qubits have the qubit after them in the same multi-qubit operation,
        // because those qubits need to get a gap row below them.
        let mut qubits_with_gap_row_below = FxHashSet::default();

        for operation in self.operations.iter() {
            for target in operation.targets.iter() {
                let qubit = target.q_id;

                if qubits_with_gap_row_below.contains(&qubit) {
                    continue;
                }

                let next_qubit = qubit + 1;

                // Check if the next qubit is also in this operation.
                if operation.targets.iter().any(|t| t.q_id == next_qubit) {
                    qubits_with_gap_row_below.insert(qubit);
                }
            }
        }

        // Initialize all qubit and classical wires
        for q in &self.qubits {
            rows.push(Row {
                wire: Wire::Qubit { q_id: q.id },
                objects: FxHashMap::default(),
                next_column: 1,
            });

            register_to_row.insert((q.id, None), rows.len() - 1);

            // If this qubit has no children, but it is in a multi-qubit operation with
            // the next qubit, we add an empty row to make room for the vertical connector.
            // We can just use a classical wire type for this row since the wire won't actually be rendered.
            let extra_rows = if qubits_with_gap_row_below.contains(&q.id) {
                cmp::max(1, q.num_children)
            } else {
                q.num_children
            };

            for i in 0..extra_rows {
                rows.push(Row {
                    wire: Wire::Classical { start_column: None },
                    objects: FxHashMap::default(),
                    next_column: 1,
                });

                register_to_row.insert((q.id, Some(i)), rows.len() - 1);
            }
        }

        for o in &self.operations {
            // Row indexes for the targets for this operation
            let targets = o
                .targets
                .iter()
                .filter_map(|reg| {
                    let reg = (reg.q_id, reg.c_id);
                    register_to_row.get(&reg).cloned()
                })
                .collect::<Vec<_>>();

            // Row indexes for the controls for this operation
            let controls = o
                .controls
                .iter()
                .filter_map(|reg| {
                    let reg = (reg.q_id, reg.c_id);
                    register_to_row.get(&reg).cloned()
                })
                .collect::<Vec<_>>();

            let mut all_rows = targets.clone();
            all_rows.extend(controls.iter());
            all_rows.sort_unstable();

            // We'll need to know the entire range of rows for this operation so we can
            // figure out the starting column and also so we can draw any
            // vertical lines that cross wires.
            let (begin, end) = all_rows.split_first().map_or((0, 0), |(first, tail)| {
                (*first, tail.last().unwrap_or(first) + 1)
            });

            // The starting column - the first available column in all
            // the rows that this operation spans.
            let column = rows[begin..end]
                .iter()
                .map(|r| r.next_column)
                .max()
                .unwrap_or(1);

            // Add the operation to the diagram
            for i in targets {
                let row = &mut rows[i];
                if matches!(row.wire, Wire::Classical { .. }) && o.is_measurement {
                    row.start_classical(column);
                } else {
                    row.add_gate(column, &o.gate, o.display_args.as_deref(), o.is_adjoint);
                };
            }

            if o.is_controlled || o.is_measurement {
                for i in controls {
                    let row = &mut rows[i];
                    if matches!(row.wire, Wire::Qubit { .. }) && o.is_measurement {
                        row.add_object(column, "M");
                    } else {
                        row.add_object(column, "●");
                    };
                }

                // If we have a control wire, draw vertical lines spanning all
                // control and target wires and crossing any in between
                // (vertical lines may overlap if there are multiple controls/targets,
                // this is ok in practice)
                for row in &mut rows[begin..end] {
                    row.add_vertical(column);
                }
            } else {
                // No control wire. Draw dashed vertical lines to connect
                // target wires if there are multiple targets
                for row in &mut rows[begin..end] {
                    row.add_dashed_vertical(column);
                }
            }
        }

        // Find the end column for the whole circuit so that
        // all qubit wires will extend until the end
        let end_column = rows
            .iter()
            .max_by_key(|r| r.next_column)
            .map_or(1, |r| r.next_column);

        // To be able to fit long-named operations, we calculate the required width for each column,
        // based on the maximum length needed for gates, where a gate X is printed as "- X -".
        let columns = (0..end_column)
            .map(|column| {
                Column::new(
                    rows.iter()
                        .filter_map(|row| row.objects.get(&column))
                        .filter_map(|object| match object {
                            CircuitObject::Object(string) => Some(string.len() + 4),
                            _ => None,
                        })
                        .chain(std::iter::once(MIN_COLUMN_WIDTH))
                        .max()
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();

        // Draw the diagram
        for row in rows {
            row.fmt(f, &columns)?;
        }

        Ok(())
    }
}
