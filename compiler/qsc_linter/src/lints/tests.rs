mod demo;

use super::{DivisionByZero, DoubleParens};
use crate::{
    linter::{self, ast::DummyWrapper},
    lints::tests::demo::LinterDemoApp,
};
use eframe::egui::ViewportBuilder;
use qsc_ast::{assigner::Assigner, mut_visit::MutVisitor, visit::Visitor};
use qsc_data_structures::line_column;
use winit::platform::windows::EventLoopBuilderExtWindows;

#[test]
fn linter_ui() {
    println!("Linter Demo");

    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder {
            title: Some("Linter Demo".to_string()),
            maximized: Some(true),
            active: Some(true),
            ..Default::default()
        },
        event_loop_builder: Some(Box::new(|event_loop_builder| {
            event_loop_builder.with_any_thread(true);
        })),
        ..Default::default()
    };

    if let Err(err) = eframe::run_native(
        "Linter Demo",
        native_options,
        Box::new(|cc| Box::new(LinterDemoApp::new(cc))),
    ) {
        {
            eprintln!("{err}");
        }
    }
}

#[test]
fn linter() {
    let source = r"
        namespace foo {
            operation RunProgram(vector : Double[]) : Unit {
                let x = ((1 + 2));
            }
        }
    ";

    run_lints(source);

    for lint in linter::drain() {
        let range = line_column::Range::from_span(line_column::Encoding::Utf8, source, &lint.span);
        let chunk = get_chunk(source, range);
        for line in chunk {
            println!("{line}");
        }
        println!("{lint:?}");
    }
}

fn run_lints(source: &str) {
    let mut parens = DoubleParens;
    let mut div_zero = DivisionByZero;

    let mut lints = [DummyWrapper(&mut parens), DummyWrapper(&mut div_zero)];

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
