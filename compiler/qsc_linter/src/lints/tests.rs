mod demo;

use crate::lints::tests::demo::LinterDemoApp;
use eframe::egui::ViewportBuilder;
use qsc_ast::{
    assigner::Assigner,
    ast::{NodeId, Package},
    mut_visit::MutVisitor,
};
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
    use crate::linter::ast::run_ast_lints;

    let source = r"
        namespace foo {
            operation RunProgram(vector : Double[]) : Unit {
                let x = ((1 + 2));
            }
        }
    ";

    let package = parse(source);
    let lints = run_ast_lints(&package);

    for lint in &lints {
        let range = line_column::Range::from_span(line_column::Encoding::Utf8, source, &lint.span);
        let chunk = get_lines(source, range);
        for line in chunk {
            println!("{line}");
        }
        println!("{lint:?}");
    }
}

fn parse(source: &str) -> Package {
    let mut package = Package {
        id: NodeId::FIRST,
        nodes: qsc_parse::top_level_nodes(source).0.into(),
        entry: None,
    };

    let mut assigner = Assigner::new();
    assigner.visit_package(&mut package);

    package
}

fn get_lines(source: &str, range: line_column::Range) -> Vec<&str> {
    let mut lines_of_interest = Vec::new();
    for (line_number, line) in source.lines().enumerate() {
        if range.start.line as usize <= line_number && line_number <= range.end.line as usize {
            lines_of_interest.push(line);
        }
    }
    lines_of_interest
}
