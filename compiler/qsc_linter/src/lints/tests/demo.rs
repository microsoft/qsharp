use std::{cell::RefCell, fmt::Write, rc::Rc};

use eframe::{egui, epaint::Vec2};
use qsc_ast::{assigner::Assigner, mut_visit::MutVisitor, visit::Visitor};
use qsc_data_structures::line_column;

use crate::{
    linter::{
        ast::{DummyWrapper, LintPass},
        Lint, LintBuffer,
    },
    lints::DoubleParens,
};

#[derive(Default)]
pub struct LinterDemoApp {
    source: String,
    reports: String,
}

impl LinterDemoApp {
    pub fn new(_ctx: &eframe::CreationContext) -> Self {
        let source = String::from(
            "namespace foo {
\toperation RunProgram(vector : Double[]) : Unit {
\t\tlet x = ((1 + 2));
\t}
}
",
        );

        let mut app = Self {
            source,
            ..Default::default()
        };

        app.compile();

        app
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.central_panel(ui);
        });
    }

    fn central_panel(&mut self, ui: &mut egui::Ui) {
        let size = ui.available_size();
        let size = Vec2 {
            x: size.x / 2.0,
            y: size.y,
        };

        ui.horizontal(|ui| {
            self.code_editor(ui, size);
            self.report_output(ui, size);
        });
    }

    fn code_editor(&mut self, ui: &mut egui::Ui, size: Vec2) {
        let res = ui.add(
            egui::TextEdit::multiline(&mut self.source)
                .code_editor()
                .desired_rows(12)
                .desired_width(size.x)
                .min_size(size),
        );

        if res.changed() {
            self.compile();
        }
    }

    fn report_output(&mut self, ui: &mut egui::Ui, size: Vec2) {
        ui.add(
            egui::TextEdit::multiline(&mut self.reports)
                .code_editor()
                .desired_rows(12)
                .desired_width(size.x)
                .min_size(size)
                .interactive(false),
        );
    }

    fn compile(&mut self) {
        self.reports.clear();
        let buffer = Rc::new(RefCell::new(LintBuffer::new()));
        run_lints(&self.source, &buffer);

        for lint in &buffer.borrow().data {
            pp(&self.source, lint, &mut self.reports);
        }
    }
}

impl eframe::App for LinterDemoApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.ui(ctx);
        ctx.request_repaint();
    }
}

fn run_lints(source: &str, buffer: &Rc<RefCell<LintBuffer>>) {
    let mut lints = [DoubleParens::new(buffer.clone())].map(DummyWrapper);

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

fn pp(source: &str, lint: &Lint, output_buffer: &mut String) {
    let range = line_column::Range::from_span(line_column::Encoding::Utf8, source, &lint.span);
    let chunk = super::get_chunk(source, range);

    writeln!(output_buffer, "{}: {}", lint.level, lint.message).unwrap();
    writeln!(
        output_buffer,
        " --> source.qs:{}:{}\n",
        range.start.line, range.start.column
    )
    .unwrap();

    if chunk.len() == 1 {
        writeln!(output_buffer, "{}", chunk.first().expect("")).unwrap();

        let mut tabs = 0;
        let mut spaces = 0;
        for c in chunk.first().expect("").chars() {
            if c == '\t' {
                tabs += 1;
            }
            if c == ' ' {
                spaces += 1;
            }
            if !(c == '\t' || c == ' ') {
                break;
            }
        }

        let col = range.start.column as usize;
        let width = (range.end.column - range.start.column) as usize;

        writeln!(
            output_buffer,
            "{x:\t>0$}{x: >1$}{x:>2$}{x:^>3$}",
            tabs,
            spaces,
            col - tabs - spaces,
            width,
            x = "",
        )
        .unwrap();
    } else {
        for line in chunk {
            writeln!(output_buffer, "{line}").unwrap();
        }
    }

    writeln!(output_buffer, "\n").unwrap();
}
