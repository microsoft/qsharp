// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, fmt::Write, ops::Not, vec};

/// Representation of a quantum circuit.
/// Implementation of `CircuitData` type from `qsharp-lang` npm package.
#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct Circuit {
    pub version: String,
    pub operations: Vec<Vec<Operation>>,
    pub qubits: Vec<Qubit>,
}

impl Circuit {
    pub const CURRENT_VERSION: &'static str = "1.0.0";

    pub fn new(operations: Vec<Vec<Operation>>, qubits: Vec<Qubit>) -> Self {
        Self {
            version: Self::CURRENT_VERSION.to_string(),
            operations,
            qubits,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
#[serde(default)]
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
    pub children: Vec<Vec<Operation>>,
}

const QUANTUM_REGISTER: usize = 0;
const CLASSICAL_REGISTER: usize = 1;

#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq, Clone)]
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

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
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
type ColumnWidthsByColumn = FxHashMap<usize, usize>;

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

    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        column_widths: &ColumnWidthsByColumn,
        end_column: usize,
    ) -> std::fmt::Result {
        // Temporary string so we can trim whitespace at the end
        let mut s = String::new();
        match &self.wire {
            Wire::Qubit { q_id: label } => {
                s.write_str(&fmt_qubit_label(*label))?;
                for column in 1..end_column {
                    let val = self.objects.get(&column);
                    let column_width = *column_widths.get(&column).unwrap_or(&MIN_COLUMN_WIDTH);
                    if let Some(v) = val {
                        s.write_str(&fmt_qubit_circuit_object(v, column_width))?;
                    } else {
                        s.write_str(&get_qubit_wire(column_width))?;
                    }
                }
            }
            Wire::Classical { start_column } => {
                for column in 0..end_column {
                    let val = self.objects.get(&column);
                    let column_width = *column_widths.get(&column).unwrap_or(&MIN_COLUMN_WIDTH);
                    if let Some(v) = val {
                        s.write_str(&fmt_classical_circuit_object(v, column_width))?;
                    } else if start_column.map_or(false, |s| column > s) {
                        s.write_str(&get_classical_wire(column_width))?;
                    } else {
                        s.write_str(&get_blank(column_width))?;
                    }
                }
            }
        }
        writeln!(f, "{}", s.trim_end())?;
        Ok(())
    }
}

const MIN_COLUMN_WIDTH: usize = 7;

/// "───────"
fn get_qubit_wire(column_width: usize) -> String {
    "─".repeat(column_width)
}

/// "═══════"
fn get_classical_wire(column_width: usize) -> String {
    "═".repeat(column_width)
}

/// "───┼───"
fn get_qubit_wire_cross(column_width: usize) -> String {
    let half_width = "─".repeat(column_width / 2);
    format!("{}┼{}", half_width, half_width)
}

/// "═══╪═══"
fn get_classical_wire_cross(column_width: usize) -> String {
    let half_width = "═".repeat(column_width / 2);
    format!("{}╪{}", half_width, half_width)
}

/// "   ╘═══"
fn get_classical_wire_start(column_width: usize) -> String {
    let first_half_width = " ".repeat(column_width / 2);
    let second_half_width = "═".repeat(column_width / 2);
    format!("{}╘{}", first_half_width, second_half_width)
}

/// "───┆───"
fn get_qubit_wire_dashed_cross(column_width: usize) -> String {
    let half_width = "─".repeat(column_width / 2);
    format!("{}┆{}", half_width, half_width)
}

/// "═══┆═══"
fn get_classical_wire_dashed_cross(column_width: usize) -> String {
    let half_width = "═".repeat(column_width / 2);
    format!("{}┆{}", half_width, half_width)
}

/// "   │   "
fn get_vertical(column_width: usize) -> String {
    let half_width = " ".repeat(column_width / 2);
    format!("{}│{}", half_width, half_width)
}

/// "   ┆   "
fn get_vertical_dashed(column_width: usize) -> String {
    let half_width = " ".repeat(column_width / 2);
    format!("{}┆{}", half_width, half_width)
}

/// "       "
fn get_blank(column_width: usize) -> String {
    " ".repeat(column_width)
}

/// "q_0  "
#[allow(clippy::doc_markdown)]
fn fmt_qubit_label(id: usize) -> String {
    let rest = MIN_COLUMN_WIDTH - 2;
    format!("q_{id: <rest$}")
}

/// "── A ──"
fn fmt_on_qubit_wire(obj: &str, column_width: usize) -> String {
    format!("{:─^column_width$}", format!(" {obj} "))
}

/// "══ A ══"
fn fmt_on_classical_wire(obj: &str, column_width: usize) -> String {
    format!("{:═^column_width$}", format!(" {obj} "))
}

fn fmt_classical_circuit_object(circuit_object: &CircuitObject, column_width: usize) -> String {
    match circuit_object {
        CircuitObject::WireCross => get_classical_wire_cross(column_width),
        CircuitObject::WireStart => get_classical_wire_start(column_width),
        CircuitObject::DashedCross => get_classical_wire_dashed_cross(column_width),
        CircuitObject::Vertical => get_vertical(column_width),
        CircuitObject::VerticalDashed => get_vertical_dashed(column_width),
        CircuitObject::Object(label) => fmt_on_classical_wire(label.as_str(), column_width),
    }
}

fn fmt_qubit_circuit_object(circuit_object: &CircuitObject, column_width: usize) -> String {
    match circuit_object {
        CircuitObject::WireCross => get_qubit_wire_cross(column_width),
        CircuitObject::WireStart => get_blank(column_width), // This should never happen
        CircuitObject::DashedCross => get_qubit_wire_dashed_cross(column_width),
        CircuitObject::Vertical => get_vertical(column_width),
        CircuitObject::VerticalDashed => get_vertical_dashed(column_width),
        CircuitObject::Object(label) => fmt_on_qubit_wire(label.as_str(), column_width),
    }
}

impl Display for Circuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rows = vec![];

        // Maintain a mapping from from Registers in the Circuit schema
        // to row in the diagram
        let mut register_to_row = FxHashMap::default();

        // Initialize all qubit and classical wires
        for q in &self.qubits {
            rows.push(Row {
                wire: Wire::Qubit { q_id: q.id },
                objects: FxHashMap::default(),
                next_column: 1,
            });

            register_to_row.insert((q.id, None), rows.len() - 1);

            for i in 0..q.num_children {
                rows.push(Row {
                    wire: Wire::Classical { start_column: None },
                    objects: FxHashMap::default(),
                    next_column: 1,
                });

                register_to_row.insert((q.id, Some(i)), rows.len() - 1);
            }
        }

        for (col_index, col) in self.operations.iter().enumerate().collect::<Vec<_>>() {
            for o in col {
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

                let column = col_index + 1;

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
        }

        // Find the end column for the whole circuit so that
        // all qubit wires will extend until the end
        let end_column = rows
            .iter()
            .max_by_key(|r| r.next_column)
            .map_or(1, |r| r.next_column);

        // To be able to fit long-named operations, we calculate the required width for each column,
        // based on the maximum length needed for gates, where a gate X is printed as "- X -".
        let column_widths = (0..end_column)
            .map(|column| {
                (
                    column,
                    rows.iter()
                        .filter_map(|row| row.objects.get(&column))
                        .filter_map(|object| match object {
                            CircuitObject::Object(string) => Some((string.len() + 4) | 1), // Column lengths need to be odd numbers
                            _ => None,
                        })
                        .chain(std::iter::once(MIN_COLUMN_WIDTH))
                        .max()
                        .unwrap(),
                )
            })
            .collect::<ColumnWidthsByColumn>();

        // Draw the diagram
        for row in rows {
            row.fmt(f, &column_widths, end_column)?;
        }

        Ok(())
    }
}

/// Converts a list of operations into a 2D grid of operations in col-row format.
/// Operations will be left-justified as much as possible in the resulting grid.
/// Children operations are recursively converted into a grid.
///
/// # Arguments
///
/// * `operations` - A vector of operations to be converted.
/// * `max_q_id` - The maximum qubit ID.
///
/// # Returns
///
/// A 2D array of operations.
pub fn operation_list_to_grid(
    mut operations: Vec<Operation>,
    max_q_id: usize,
) -> Vec<Vec<Operation>> {
    for op in &mut operations {
        if op.children.len() == 1 {
            op.children = operation_list_to_grid(op.children.remove(0), max_q_id);
        }
    }
    remove_padding(operation_list_to_padded_array(operations, max_q_id))
}

/// Converts a list of operations into a padded 2D array of operations.
///
/// # Arguments
///
/// * `operations` - A vector of operations to be converted.
/// * `max_q_id` - The maximum qubit ID.
///
/// # Returns
///
/// A 2D vector of optional operations padded with `None`.
fn operation_list_to_padded_array(
    operations: Vec<Operation>,
    max_q_id: usize,
) -> Vec<Vec<Option<Operation>>> {
    if operations.is_empty() {
        return vec![];
    }

    let grouped_ops = group_operations(&operations, max_q_id);
    let aligned_ops = transform_to_col_row(align_ops(grouped_ops));

    // Need to convert to optional operations so we can
    // take operations out without messing up the indexing
    let mut operations = operations.into_iter().map(Some).collect::<Vec<_>>();
    aligned_ops
        .into_iter()
        .map(|col| {
            col.into_iter()
                .map(|op_idx| op_idx.and_then(|idx| operations[idx].take()))
                .collect()
        })
        .collect()
}

/// Removes padding (`None` values) from a 2D array of operations.
///
/// # Arguments
///
/// * `operations` - A 2D vector of optional operations padded with `None`.
///
/// # Returns
///
/// A 2D vector of operations without `None` values.
fn remove_padding(operations: Vec<Vec<Option<Operation>>>) -> Vec<Vec<Operation>> {
    operations
        .into_iter()
        .map(|col| col.into_iter().flatten().collect())
        .collect()
}

/// Transforms a row-col 2D array into an equivalent col-row 2D array.
///
/// # Arguments
///
/// * `aligned_ops` - A 2D vector of optional usize values in row-col format.
///
/// # Returns
///
/// A 2D vector of optional usize values in col-row format.
fn transform_to_col_row(aligned_ops: Vec<Vec<Option<usize>>>) -> Vec<Vec<Option<usize>>> {
    if aligned_ops.is_empty() {
        return vec![];
    }

    let num_rows = aligned_ops.len();
    let num_cols = aligned_ops.iter().map(|row| row.len()).max().unwrap_or(0);

    let mut col_row_array = vec![vec![None; num_rows]; num_cols];

    for (row, row_data) in aligned_ops.into_iter().enumerate() {
        for (col, value) in row_data.into_iter().enumerate() {
            col_row_array[col][row] = value;
        }
    }

    col_row_array
}

/// Groups operations by their respective registers.
///
/// # Arguments
///
/// * `operations` - A slice of operations to be grouped.
/// * `max_q_id` - The maximum qubit ID.
///
/// # Returns
///
/// A 2D vector of indices where `groupedOps[i][j]` is the index of the operations
/// at register `i` and column `j` (not yet aligned/padded).
fn group_operations(operations: &[Operation], max_q_id: usize) -> Vec<Vec<usize>> {
    let end_q_id = max_q_id + 1; // Add one so that it is an "end" index
    let mut grouped_ops = vec![vec![]; end_q_id];

    for (instr_idx, op) in operations.iter().enumerate() {
        let ctrls = &op.controls;
        let q_regs: Vec<_> = ctrls
            .iter()
            .chain(&op.targets)
            .filter(|reg| reg.r#type == QUANTUM_REGISTER)
            .collect();
        let q_reg_idx_list: Vec<_> = q_regs.iter().map(|reg| reg.q_id).collect();
        let cls_controls: Vec<_> = ctrls
            .iter()
            .filter(|reg| reg.r#type == CLASSICAL_REGISTER)
            .collect();
        let is_classically_controlled = !cls_controls.is_empty();

        if !is_classically_controlled && q_regs.is_empty() {
            continue;
        }

        let min_reg_idx = if is_classically_controlled {
            0
        } else {
            *q_reg_idx_list.iter().min().unwrap()
        };
        let max_reg_idx = if is_classically_controlled {
            end_q_id - 1
        } else {
            *q_reg_idx_list.iter().max().unwrap()
        };

        for reg_ops in grouped_ops
            .iter_mut()
            .take(max_reg_idx + 1)
            .skip(min_reg_idx)
        {
            reg_ops.push(instr_idx);
        }
    }

    grouped_ops
}

/// Aligns operations by padding registers with `None` to make sure that multiqubit
/// gates are in the same column.
///
/// # Arguments
///
/// * `ops` - A 2D vector of usize values representing the operations.
///
/// # Returns
///
/// A 2D vector of optional usize values representing the aligned operations.
fn align_ops(ops: Vec<Vec<usize>>) -> Vec<Vec<Option<usize>>> {
    let mut max_num_ops = ops.iter().map(|reg_ops| reg_ops.len()).max().unwrap_or(0);
    let mut col = 0;
    let mut padded_ops: Vec<Vec<Option<usize>>> = ops
        .into_iter()
        .map(|reg_ops| reg_ops.into_iter().map(Some).collect())
        .collect();

    while col < max_num_ops {
        for reg_idx in 0..padded_ops.len() {
            if padded_ops[reg_idx].len() <= col {
                continue;
            }

            // Represents the gate at padded_ops[reg_idx][col]
            let op_idx = padded_ops[reg_idx][col];

            // The vec of where in each register the gate appears
            let targets_pos: Vec<_> = padded_ops
                .iter()
                .map(|reg_ops| reg_ops.iter().position(|&x| x == op_idx))
                .collect();
            // The maximum column index of the gate in the target registers
            let gate_max_col = targets_pos
                .iter()
                .filter_map(|&pos| pos)
                .max()
                .unwrap_or(usize::MAX);

            if col < gate_max_col {
                padded_ops[reg_idx].insert(col, None);
                max_num_ops = max_num_ops.max(padded_ops[reg_idx].len());
            }
        }
        col += 1;
    }

    padded_ops
}
