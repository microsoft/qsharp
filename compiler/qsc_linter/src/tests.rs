use qsc_ast::{
    assigner::Assigner,
    ast::{NodeId, Package},
    mut_visit::MutVisitor,
};
use qsc_data_structures::{language_features::LanguageFeatures, line_column};

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
        nodes: qsc_parse::top_level_nodes(source, LanguageFeatures::default())
            .0
            .into(),
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
