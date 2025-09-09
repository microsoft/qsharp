// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::{
    cmp::{self, max},
    fmt::{Display, Write},
    ops::Not,
    vec,
};

/// Current format version.
pub const CURRENT_VERSION: usize = 1;

/// Representation of a quantum circuit group.
#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct CircuitGroup {
    pub circuits: Vec<Circuit>,
    pub version: usize,
}

impl Display for CircuitGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for circuit in &self.circuits {
            writeln!(f, "{circuit}")?;
        }
        Ok(())
    }
}

/// Representation of a quantum circuit.
#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct Circuit {
    pub qubits: Vec<Qubit>,
    #[serde(rename = "componentGrid")]
    pub component_grid: ComponentGrid,
}

/// Type alias for a grid of components.
pub type ComponentGrid = Vec<ComponentColumn>;

/// Representation of a column in the component grid.
#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct ComponentColumn {
    pub components: Vec<Component>,
}

/// Union type for components.
pub type Component = Operation;

/// Union type for operations.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "kind")]
pub enum Operation {
    #[serde(rename = "measurement")]
    Measurement(Measurement),
    #[serde(rename = "unitary")]
    Unitary(Unitary),
    #[serde(rename = "ket")]
    Ket(Ket),
}

impl Operation {
    /// Returns the gate name of the operation.
    #[must_use]
    pub fn gate(&self) -> String {
        match self {
            Operation::Measurement(m) => m.gate.clone(),
            Operation::Unitary(u) => u.gate.clone(),
            #[allow(clippy::unicode_not_nfc)]
            Operation::Ket(k) => format!("|{}〉", k.gate),
        }
    }

    /// Returns the arguments for the operation.
    #[must_use]
    pub fn args(&self) -> Vec<String> {
        match self {
            Operation::Measurement(m) => m.args.clone(),
            Operation::Unitary(u) => u.args.clone(),
            Operation::Ket(k) => k.args.clone(),
        }
    }

    /// Returns the children for the operation.
    #[must_use]
    pub fn children(&self) -> &ComponentGrid {
        match self {
            Operation::Measurement(m) => &m.children,
            Operation::Unitary(u) => &u.children,
            Operation::Ket(k) => &k.children,
        }
    }

    /// Returns if the operation is a controlled operation.
    #[must_use]
    pub fn is_controlled(&self) -> bool {
        match self {
            Operation::Measurement(_) | Operation::Ket(_) => false,
            Operation::Unitary(u) => !u.controls.is_empty(),
        }
    }

    /// Returns if the operation is a measurement operation.
    #[must_use]
    pub fn is_measurement(&self) -> bool {
        match self {
            Operation::Measurement(_) => true,
            Operation::Unitary(_) | Operation::Ket(_) => false,
        }
    }

    /// Returns if the operation is an adjoint operation.
    #[must_use]
    pub fn is_adjoint(&self) -> bool {
        match self {
            Operation::Measurement(_) | Operation::Ket(_) => false,
            Operation::Unitary(u) => u.is_adjoint,
        }
    }
}

/// Representation of a measurement operation.
#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct Measurement {
    pub gate: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub children: ComponentGrid,
    pub qubits: Vec<Register>,
    pub results: Vec<Register>,
}

/// Representation of a unitary operation.
#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct Unitary {
    pub gate: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub children: ComponentGrid,
    pub targets: Vec<Register>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub controls: Vec<Register>,
    #[serde(rename = "isAdjoint")]
    #[serde(skip_serializing_if = "Not::not")]
    #[serde(default)]
    pub is_adjoint: bool,
}

/// Representation of a gate that will set the target to a specific state.
#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct Ket {
    pub gate: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub children: ComponentGrid,
    pub targets: Vec<Register>,
}

#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq, Clone)]
pub struct Register {
    pub qubit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<usize>,
}

impl Register {
    #[must_use]
    pub fn quantum(qubit_id: usize) -> Self {
        Self {
            qubit: qubit_id,
            result: None,
        }
    }

    #[must_use]
    pub fn classical(qubit_id: usize, result_id: usize) -> Self {
        Self {
            qubit: qubit_id,
            result: Some(result_id),
        }
    }

    #[must_use]
    pub fn is_classical(&self) -> bool {
        self.result.is_some()
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct Qubit {
    pub id: usize,
    #[serde(rename = "numResults")]
    #[serde(default)]
    pub num_results: usize,
}

#[derive(Clone, Debug, Copy, Default)]
pub struct Config {
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

    fn add_gate(&mut self, column: usize, gate: &str, args: &[String], is_adjoint: bool) {
        let mut gate_label = String::new();
        gate_label.push_str(gate);
        if is_adjoint {
            gate_label.push('\'');
        }

        let args_without_metadata = args
            .iter()
            .filter(|arg| !arg.starts_with("metadata="))
            .cloned()
            .collect::<Vec<_>>();
        if !args_without_metadata.is_empty() {
            let args = args_without_metadata.join(", ");
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
            CircuitObject::WireStart // This should never happen
            | CircuitObject::Blank => BLANK,
            CircuitObject::Wire => QUBIT_WIRE,
            CircuitObject::WireCross => QUBIT_WIRE_CROSS,
            CircuitObject::DashedCross => QUBIT_WIRE_DASHED_CROSS,
            CircuitObject::Vertical => VERTICAL,
            CircuitObject::VerticalDashed => VERTICAL_DASHED,
            CircuitObject::Object(_) => unreachable!("This case is covered in the early return."),
        };

        self.expand_template(&template)
    }
}

impl Display for Circuit {
    /// Formats the circuit into a diagram.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rows = vec![];

        // Maintain a mapping from from Registers in the Circuit schema
        // to row in the diagram
        let mut register_to_row = FxHashMap::default();

        // Keep track of which qubits have the qubit after them in the same multi-qubit operation,
        // because those qubits need to get a gap row below them.
        let mut qubits_with_gap_row_below = FxHashSet::default();

        // Identify qubits that require gap rows
        self.identify_qubits_with_gap_rows(&mut qubits_with_gap_row_below);

        // Initialize rows for qubits and classical wires
        self.initialize_rows(&mut rows, &mut register_to_row, &qubits_with_gap_row_below);

        // Add operations to the diagram
        Self::add_operations_to_diagram(1, &self.component_grid, &mut rows, &register_to_row);

        // Finalize the diagram by extending wires and formatting columns
        let columns = finalize_columns(&rows);

        // Draw the diagram
        for row in rows {
            row.fmt(f, &columns)?;
        }

        Ok(())
    }
}

impl Circuit {
    /// Identifies qubits that require gap rows for multi-qubit operations.
    fn identify_qubits_with_gap_rows(&self, qubits_with_gap_row_below: &mut FxHashSet<usize>) {
        for col in &self.component_grid {
            for op in &col.components {
                let targets = match op {
                    Operation::Measurement(m) => &m.qubits,
                    Operation::Unitary(u) => &u.targets,
                    Operation::Ket(k) => &k.targets,
                };
                for target in targets {
                    let qubit = target.qubit;

                    if qubits_with_gap_row_below.contains(&qubit) {
                        continue;
                    }

                    let next_qubit = qubit + 1;

                    // Check if the next qubit is also in this operation.
                    if targets.iter().any(|t| t.qubit == next_qubit) {
                        qubits_with_gap_row_below.insert(qubit);
                    }
                }
            }
        }
    }

    /// Initializes rows for qubits and classical wires.
    fn initialize_rows(
        &self,
        rows: &mut Vec<Row>,
        register_to_row: &mut FxHashMap<(usize, Option<usize>), usize>,
        qubits_with_gap_row_below: &FxHashSet<usize>,
    ) {
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
                cmp::max(1, q.num_results)
            } else {
                q.num_results
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
    }

    /// Adds operations to the diagram.
    fn add_operations_to_diagram(
        start_column: usize,
        component_grid: &ComponentGrid,
        rows: &mut [Row],
        register_to_row: &FxHashMap<(usize, Option<usize>), usize>,
    ) {
        let mut column = start_column;
        let mut next_column = start_column;
        for col in component_grid {
            for op in &col.components {
                let targets = get_row_indexes(op, register_to_row, true);
                let controls = get_row_indexes(op, register_to_row, false);

                let mut all_rows = targets.clone();
                all_rows.extend(controls.iter());
                all_rows.sort_unstable();

                // We'll need to know the entire range of rows for this operation so we can
                // figure out the starting column and also so we can draw any
                // vertical lines that cross wires.
                let (begin, end) = all_rows.split_first().map_or((0, 0), |(first, tail)| {
                    (*first, tail.last().unwrap_or(first) + 1)
                });

                let children = op.children();
                if children.is_empty() {
                    add_operation_to_rows(op, rows, &targets, &controls, column, begin, end);
                    next_column = max(next_column, column + 1);
                } else {
                    let mut offset = 0;
                    add_operation_box_start_to_rows(
                        op,
                        rows,
                        &targets,
                        &controls,
                        column + offset,
                        begin,
                        end,
                    );
                    offset += 2;
                    Self::add_operations_to_diagram(
                        column + offset,
                        children,
                        rows,
                        register_to_row,
                    );
                    offset += children.len();
                    add_operation_box_end_to_rows(op, rows, &targets, column + offset);
                    offset += 1;
                    next_column = max(next_column, column + offset);
                }
            }
            column = next_column;
        }
    }
}

/// Adds a single operation to the rows.
fn add_operation_to_rows(
    operation: &Operation,
    rows: &mut [Row],
    targets: &[usize],
    controls: &[usize],
    column: usize,
    begin: usize,
    end: usize,
) {
    for i in targets {
        let row = &mut rows[*i];
        if matches!(row.wire, Wire::Classical { .. })
            && matches!(operation, Operation::Measurement(_))
        {
            row.start_classical(column);
        } else {
            row.add_gate(
                column,
                &operation.gate(),
                &operation.args(),
                operation.is_adjoint(),
            );
        }
    }

    if operation.is_controlled() || operation.is_measurement() {
        for i in controls {
            let row = &mut rows[*i];
            if matches!(row.wire, Wire::Qubit { .. }) && operation.is_measurement() {
                row.add_object(column, "M");
            } else {
                row.add_object(column, "●");
            }
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

fn add_operation_box_start_to_rows(
    operation: &Operation,
    rows: &mut [Row],
    targets: &[usize],
    controls: &[usize],
    column: usize,
    begin: usize,
    end: usize,
) {
    assert!(
        !operation.children().is_empty(),
        "must only be called for an operation with children"
    );
    for i in targets {
        let row = &mut rows[*i];
        row.add_object(column, "[[");

        let mut gate_label = String::new();
        gate_label.push('[');
        gate_label.push_str(&operation.gate());
        if operation.is_adjoint() {
            gate_label.push('\'');
        }
        let args = operation.args();
        let args_without_metadata = args
            .iter()
            .filter(|arg| !arg.starts_with("metadata="))
            .cloned()
            .collect::<Vec<_>>();
        if !args_without_metadata.is_empty() {
            let args = args_without_metadata.join(", ");
            let _ = write!(&mut gate_label, "({args})");
        }
        gate_label.push(']');

        row.add_object(column + 1, gate_label.as_str());
    }

    if operation.is_controlled() || operation.is_measurement() {
        for i in controls {
            let row = &mut rows[*i];
            if matches!(row.wire, Wire::Qubit { .. }) && operation.is_measurement() {
                row.add_object(column + 1, "M");
            } else {
                row.add_object(column + 1, "●");
            }
        }

        // If we have a control wire, draw vertical lines spanning all
        // control and target wires and crossing any in between
        // (vertical lines may overlap if there are multiple controls/targets,
        // this is ok in practice)
        for row in &mut rows[begin..end] {
            row.add_vertical(column + 1);
        }
    } else {
        // No control wire. Draw dashed vertical lines to connect
        // target wires if there are multiple targets
        for row in &mut rows[begin..end] {
            row.add_dashed_vertical(column + 1);
        }
    }
}

fn add_operation_box_end_to_rows(
    operation: &Operation,
    rows: &mut [Row],
    targets: &[usize],
    column: usize,
) {
    assert!(
        !operation.children().is_empty(),
        "must only be called for an operation with children"
    );
    for i in targets {
        let row = &mut rows[*i];
        row.add_object(column, "]]");
    }
}

/// Finalizes the columns by calculating their widths.
fn finalize_columns(rows: &[Row]) -> Vec<Column> {
    // Find the end column for the whole circuit so that
    // all qubit wires will extend until the end
    let end_column = rows
        .iter()
        .max_by_key(|r| r.next_column)
        .map_or(1, |r| r.next_column);

    // To be able to fit long-named operations, we calculate the required width for each column,
    // based on the maximum length needed for gates, where a gate X is printed as "- X -".
    (0..end_column)
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
                    .expect("Column width should be at least 1"),
            )
        })
        .collect()
}

/// Gets the row indexes for the targets or controls of an operation.
fn get_row_indexes(
    operation: &Operation,
    register_to_row: &FxHashMap<(usize, Option<usize>), usize>,
    is_target: bool,
) -> Vec<usize> {
    let registers = match operation {
        Operation::Measurement(m) => {
            if is_target {
                &m.results
            } else {
                &m.qubits
            }
        }
        Operation::Unitary(u) => {
            if is_target {
                &u.targets
            } else {
                &u.controls
            }
        }
        Operation::Ket(k) => {
            if is_target {
                &k.targets
            } else {
                &vec![]
            }
        }
    };

    registers
        .iter()
        .filter_map(|reg| {
            let reg = (reg.qubit, reg.result);
            register_to_row.get(&reg).copied()
        })
        .collect()
}

/// Converts a list of operations into a 2D grid of operations in col-row format.
/// Operations will be left-justified as much as possible in the resulting grid.
/// Children operations are recursively converted into a grid.
///
/// # Arguments
///
/// * `operations` - A vector of operations to be converted.
/// * `num_qubits` - The number of qubits in the circuit.
///
/// # Returns
///
/// A component grid representing the operations.
#[must_use]
pub fn operation_list_to_grid(operations: &[Operation], num_qubits: usize) -> ComponentGrid {
    let mut operations = collapse_repetition(operations);
    for op in &mut operations {
        // The children data structure is a grid, so checking if it is
        // length 1 is actually checking if it has a single column,
        // or in other words, we are checking if its children are in a single list.
        // If the operation has children in a single list, it needs to be converted to a grid.
        // If it was already converted to a grid, but the grid was still a single list,
        // then doing it again won't effect anything.
        if op.children().len() == 1 {
            match op {
                Operation::Measurement(m) => {
                    let child_vec = m.children.remove(0).components; // owns
                    m.children = operation_list_to_grid(&child_vec, num_qubits);
                }
                Operation::Unitary(u) => {
                    let child_vec = u.children.remove(0).components;
                    u.children = operation_list_to_grid(&child_vec, num_qubits);
                }
                Operation::Ket(k) => {
                    let child_vec = k.children.remove(0).components;
                    k.children = operation_list_to_grid(&child_vec, num_qubits);
                }
            }
        }
    }

    // Convert the operations into a component grid
    let mut component_grid = vec![];
    for col in remove_padding(operation_list_to_padded_array(operations, num_qubits)) {
        let column = ComponentColumn { components: col };
        component_grid.push(column);
    }
    component_grid
}

fn make_repeated_parent(base: &Operation, count: usize) -> Operation {
    debug_assert!(count > 1);
    let mut parent = base.clone();
    let child_columns: ComponentGrid = (0..count)
        .map(|_| ComponentColumn {
            components: vec![base.clone()],
        })
        .collect();
    match &mut parent {
        Operation::Measurement(m) => {
            m.children = child_columns;
            m.gate = format!("{}({})", m.gate, count);
        }
        Operation::Unitary(u) => {
            u.children = child_columns;
            u.gate = format!("{}({})", u.gate, count);
            // Merge targets and controls into targets; clear controls
            let mut seen: FxHashSet<(usize, Option<usize>)> = FxHashSet::default();
            let mut merged: Vec<Register> = Vec::new();
            for r in u.targets.iter().chain(u.controls.iter()) {
                let key = (r.qubit, r.result);
                if seen.insert(key) {
                    merged.push(r.clone());
                }
            }
            u.targets = merged;
            u.controls.clear();
        }
        Operation::Ket(k) => {
            k.children = child_columns;
            k.gate = format!("{}({})", k.gate, count);
        }
    }
    parent
}

#[allow(clippy::too_many_lines)]
fn collapse_repetition(operations: &[Operation]) -> Vec<Operation> {
    // Extended: detect repeating motifs of length > 1 as well (e.g. A B A B A B -> (A B)(3)).
    // Strategy: scan list; for each start index find longest total repeated sequence comprising
    // repeats (>1) of a motif whose operations are all the same variant type. Prefer the match
    // with the greatest total collapsed length; tie-breaker smaller motif length.
    let len = operations.len();
    let mut i = 0;
    let mut result: Vec<Operation> = Vec::new();

    while i < len {
        // Determine best motif at position i.
        let remaining = len - i;
        let mut best_motif_len = 1usize; // at least single op repetition handled
        let mut best_repeats = 1usize;
        let max_motif_len = (remaining / 2).max(1); // need at least 2 motifs to repeat

        for motif_len in 1..=max_motif_len {
            // Quick break: if motif_len already larger than remaining/ best potential *2
            if motif_len * 2 > remaining {
                break;
            }

            // Candidate motif slice
            let motif = &operations[i..i + motif_len];
            // Restrict to same variant type for all ops in motif
            let first_discriminant = std::mem::discriminant(&motif[0]);
            if motif
                .iter()
                .any(|op| std::mem::discriminant(op) != first_discriminant)
            {
                continue;
            }

            // Count repeats
            let mut repeats = 1usize;
            'outer: loop {
                let start_next = i + repeats * motif_len;
                let end_next = start_next + motif_len;
                if end_next > len {
                    break;
                }
                for k in 0..motif_len {
                    if operations[i + k] != operations[start_next + k] {
                        break 'outer;
                    }
                }
                repeats += 1;
            }

            if repeats > 1 {
                let total = repeats * motif_len;
                let best_total = best_repeats * best_motif_len;
                if total > best_total || (total == best_total && motif_len < best_motif_len) {
                    best_motif_len = motif_len;
                    best_repeats = repeats;
                }
            }
        }

        if best_repeats > 1 {
            // Build parent from first operation in motif.
            let base = &operations[i];
            if best_motif_len == 1 {
                result.push(make_repeated_parent(base, best_repeats));
            } else {
                // Build child grid: replicate the whole repeated sequence (motif * repeats)
                let repeated_slice = &operations[i..i + best_repeats * best_motif_len];
                let mut parent = base.clone();
                // Gather motif gates for label
                let motif_gates: Vec<String> = operations[i..i + best_motif_len]
                    .iter()
                    .map(|op| match op {
                        Operation::Measurement(m) => m.gate.clone(),
                        Operation::Unitary(u) => u.gate.clone(),
                        Operation::Ket(k) => k.gate.clone(),
                    })
                    .collect();
                let mut label_prefix = motif_gates.join(" ");
                if label_prefix.chars().count() > 5 {
                    let truncated: String = label_prefix.chars().take(5).collect();
                    label_prefix = format!("{truncated}...");
                }
                let mut child_columns: ComponentGrid =
                    Vec::with_capacity(best_repeats * best_motif_len);
                for op in repeated_slice {
                    child_columns.push(ComponentColumn {
                        components: vec![op.clone()],
                    });
                }
                match &mut parent {
                    Operation::Measurement(m) => {
                        m.children = child_columns;
                        m.gate = format!("{label_prefix}({best_repeats})");
                    }
                    Operation::Unitary(u) => {
                        u.children = child_columns;
                        u.gate = format!("{label_prefix}({best_repeats})");
                        // Union of targets and controls across all repeated operations into targets only
                        let mut seen: FxHashSet<(usize, Option<usize>)> = FxHashSet::default();
                        let mut merged = Vec::new();
                        for op in repeated_slice {
                            if let Operation::Unitary(child) = op {
                                for r in child.targets.iter().chain(child.controls.iter()) {
                                    let key = (r.qubit, r.result);
                                    if seen.insert(key) {
                                        merged.push(r.clone());
                                    }
                                }
                            }
                        }
                        u.targets = merged;
                        u.controls.clear();
                    }
                    Operation::Ket(k) => {
                        k.children = child_columns;
                        k.gate = format!("{label_prefix}({best_repeats})");
                        // Union of targets across all repeated operations
                        let mut tgt_seen: FxHashSet<(usize, Option<usize>)> = FxHashSet::default();
                        let mut new_targets = Vec::new();
                        for op in repeated_slice {
                            if let Operation::Ket(child) = op {
                                for r in &child.targets {
                                    let key = (r.qubit, r.result);
                                    if tgt_seen.insert(key) {
                                        new_targets.push(r.clone());
                                    }
                                }
                            }
                        }
                        k.targets = new_targets;
                    }
                }
                result.push(parent);
            }
            i += best_repeats * best_motif_len;
        } else {
            // No pattern; push single op
            result.push(operations[i].clone());
            i += 1;
        }
    }

    result
}

/// Converts a list of operations into a padded 2D array of operations.
///
/// # Arguments
///
/// * `operations` - A vector of operations to be converted.
/// * `num_qubits` - The number of qubits in the circuit.
///
/// # Returns
///
/// A 2D vector of optional operations padded with `None`.
fn operation_list_to_padded_array(
    operations: Vec<Operation>,
    num_qubits: usize,
) -> Vec<Vec<Option<Operation>>> {
    if operations.is_empty() {
        return vec![];
    }

    let grouped_ops = group_operations(&operations, num_qubits);
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
    let num_cols = aligned_ops
        .iter()
        .map(std::vec::Vec::len)
        .max()
        .unwrap_or(0);

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
/// * `num_qubits` - The number of qubits in the circuit.
///
/// # Returns
///
/// A 2D vector of indices where `groupedOps[i][j]` is the index of the operations
/// at register `i` and column `j` (not yet aligned/padded).
fn group_operations(operations: &[Operation], num_qubits: usize) -> Vec<Vec<usize>> {
    let mut grouped_ops = vec![vec![]; num_qubits];

    let max_q_id = match num_qubits {
        0 => 0,
        _ => num_qubits - 1,
    };

    for (instr_idx, op) in operations.iter().enumerate() {
        let ctrls = match op {
            Operation::Measurement(m) => &m.qubits,
            Operation::Unitary(u) => &u.controls,
            Operation::Ket(_) => &vec![],
        };
        let targets = match op {
            Operation::Measurement(m) => &m.results,
            Operation::Unitary(u) => &u.targets,
            Operation::Ket(k) => &k.targets,
        };
        let q_regs: Vec<_> = ctrls
            .iter()
            .chain(targets)
            .filter(|reg| !reg.is_classical())
            .collect();
        let q_reg_idx_list: Vec<_> = q_regs.iter().map(|reg| reg.qubit).collect();
        let cls_controls: Vec<_> = ctrls.iter().filter(|reg| reg.is_classical()).collect();
        let is_classically_controlled = !cls_controls.is_empty();

        if !is_classically_controlled && q_regs.is_empty() {
            continue;
        }

        let (min_reg_idx, max_reg_idx) = if is_classically_controlled {
            (0, max_q_id)
        } else {
            q_reg_idx_list
                .into_iter()
                .fold(None, |acc, x| match acc {
                    None => Some((x, x)),
                    Some((min, max)) => Some((min.min(x), max.max(x))),
                })
                .unwrap_or((0, max_q_id))
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
    let mut max_num_ops = ops.iter().map(std::vec::Vec::len).max().unwrap_or(0);
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
