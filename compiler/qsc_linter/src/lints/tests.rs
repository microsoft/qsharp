use std::{cell::RefCell, rc::Rc};

use qsc_ast::{assigner::Assigner, mut_visit::MutVisitor, visit::Visitor};

use crate::linter::{
    ast::{DummyWrapper, LintPass},
    LintBuffer,
};

use qsc_data_structures::line_column;

use super::DoubleParens;

#[test]
fn linter() {
    let source = r"
        namespace foo {
            operation RunProgram(vector : Double[]) : Unit {
                let x = ((1 + 2));
            }
        }
    ";

    let buffer = Rc::new(RefCell::new(LintBuffer::new()));
    run_lints(source, &buffer);

    for lint in &buffer.borrow().data {
        let range = line_column::Range::from_span(line_column::Encoding::Utf8, source, &lint.span);
        let chunk = get_chunk(source, range);
        for line in chunk {
            println!("{line}");
        }
        println!("{lint:?}");
    }
}

fn run_lints(source: &str, buffer: &Rc<RefCell<LintBuffer>>) {
    let mut lints = [
        DoubleParens::new(buffer.clone()),
        DoubleParens::new(buffer.clone()),
        DoubleParens::new(buffer.clone()),
    ]
    .map(DummyWrapper);

    let (mut namespaces, _) = qsc_parse::namespaces(source);
    let mut assigner = Assigner::new();

    for namespace in &mut namespaces {
        assigner.visit_namespace(namespace);
    }

    for namespace in &namespaces {
        for lint in &mut lints {
            lint.visit_namespace(namespace);
        }
    }
}

fn get_chunk(source: &str, range: line_column::Range) -> Vec<&str> {
    let mut lines_of_interest = Vec::new();
    for (line_number, line) in source.lines().enumerate() {
        if range.start.line as usize <= line_number && line_number <= range.end.line as usize {
            lines_of_interest.push(line);
        }
    }
    lines_of_interest
}
