use std::fmt::Write;

use eframe::{
    egui::{self, RichText},
    epaint::{Color32, Vec2},
};
use qsc_ast::{assigner::Assigner, mut_visit::MutVisitor, visit::Visitor};
use qsc_data_structures::line_column;

use crate::{
    linter::{self, ast::AstLintWrapper, Lint, LintLevel},
    lints::ast::{DivisionByZero, DoubleParens},
};

#[derive(Default)]
pub struct LinterDemoApp {
    source: String,
    lints: Vec<Lint>,
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
            self.report_output_colored(ui, size);
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

    fn report_output_colored(&mut self, ui: &mut egui::Ui, _size: Vec2) {
        ui.vertical(|ui| {
            egui::scroll_area::ScrollArea::vertical().show(ui, |ui| {
                for lint in &self.lints {
                    pp_to_ui(&self.source, lint, ui);
                }
            });
        });
    }

    fn compile(&mut self) {
        run_lints(&self.source);
        self.lints = linter::drain().collect();
    }
}

impl eframe::App for LinterDemoApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.ui(ctx);
        ctx.request_repaint();
    }
}

fn run_lints(source: &str) {
    let mut parens = DoubleParens;
    let mut div_zero = DivisionByZero;

    let mut lints = [AstLintWrapper(&mut parens), AstLintWrapper(&mut div_zero)];

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

fn _pp(source: &str, lint: &Lint, output_buffer: &mut String) {
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

fn pp_to_ui(source: &str, lint: &Lint, ui: &mut egui::Ui) {
    let range = line_column::Range::from_span(line_column::Encoding::Utf8, source, &lint.span);
    let chunk = super::get_chunk(source, range);

    let lint_color = match lint.level {
        LintLevel::Allow => ui.visuals().text_color(),
        LintLevel::Warn | LintLevel::ForceWarn => Color32::YELLOW,
        LintLevel::Deny | LintLevel::ForceDeny => Color32::RED,
    };
    let margin_color = Color32::LIGHT_BLUE;

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            colored_code(ui, lint.level.to_string(), lint_color);
            code(ui, format!(": {}", lint.message));
        });

        ui.horizontal(|ui| {
            colored_code(ui, " --> ", margin_color);
            code(
                ui,
                format!("source.qs:{}:{}", range.start.line, range.start.column),
            );
        });

        ui.spacing();

        if chunk.len() == 1 {
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

            let underline = format!(
                "{x:\t>0$}{x: >1$}{x:>2$}{x:^>3$}\n",
                tabs,
                spaces,
                col - tabs - spaces,
                width,
                x = "",
            );

            code(ui, (*chunk.first().expect("")).to_string());
            colored_code(ui, underline, lint_color);
        } else {
            for line in chunk {
                code(ui, format!("{line}\n"));
            }
        }

        ui.spacing();
    });
}

fn code(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    ui.label(
        RichText::new(text)
            .monospace()
            .background_color(ui.style().visuals.panel_fill),
    )
}

fn colored_code(ui: &mut egui::Ui, text: impl Into<String>, color: Color32) -> egui::Response {
    ui.colored_label(
        color,
        RichText::new(text)
            .monospace()
            .background_color(ui.style().visuals.panel_fill),
    )
}
