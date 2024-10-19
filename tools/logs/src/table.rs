use crate::{entries::LogEntry, gui::Application};
use egui::{RichText, TextStyle, TextWrapMode};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct LogsTable {
    scroll_to_row_slider: usize,
    scroll_to_row: Option<usize>,
    selection: std::collections::HashSet<usize>,
    search: String,
}

impl Default for LogsTable {
    fn default() -> Self {
        Self {
            scroll_to_row_slider: 0,
            scroll_to_row: None,
            selection: Default::default(),
            search: Default::default(),
        }
    }
}

impl LogsTable {
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui, app: &mut Application) {
        let reset = false;

        ui.separator();

        let body_text_size = TextStyle::Body.resolve(ui.style()).size;
        use egui_extras::{Size, StripBuilder};
        StripBuilder::new(ui)
            .size(Size::remainder().at_least(100.0))
            .size(Size::exact(body_text_size))
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    let entries = app.entries();

                    ui.add(
                        egui::TextEdit::singleline(&mut app.search)
                            .hint_text("Write something here"),
                    );
                    ui.end_row();

                    if entries.is_empty() {
                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            ui.with_layout(
                                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                                |ui| {
                                    ui.label(
                                        RichText::new("No logs to display")
                                            .size(body_text_size + 10.0),
                                    );
                                    return;
                                },
                            );
                        });
                    } else {
                        self.table_ui(ui, reset, entries, app);
                    }
                });
            });
    }

    fn table_ui(
        &mut self,
        ui: &mut egui::Ui,
        reset: bool,
        entries: Vec<LogEntry>,
        app: &mut Application,
    ) {
        use egui_extras::{Column, TableBuilder};

        let text_height = TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        let available_height = ui.available_height();
        let mut table = TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::remainder())
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);

        table = table.sense(egui::Sense::click());

        if let Some(row_index) = self.scroll_to_row.take() {
            table = table.scroll_to_row(row_index, None);
        }

        if reset {
            table.reset();
        }

        let rows = entries.len();
        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    egui::Sides::new().show(
                        ui,
                        |ui| {
                            ui.strong("Row");
                        },
                        |ui| {
                            app.reversed ^=
                                ui.button(if app.reversed { "⬆" } else { "⬇" }).clicked();
                        },
                    );
                });
                header.col(|ui| {
                    ui.strong("Time");
                });
                header.col(|ui| {
                    ui.strong("Level");
                });
                header.col(|ui| {
                    ui.strong("Module");
                });
                header.col(|ui| {
                    ui.strong("File");
                });
                header.col(|ui| {
                    ui.strong("Message");
                });
            })
            .body(|body| {
                body.rows(text_height, rows, |mut row| {
                    let row_index = if app.reversed {
                        rows - 1 - row.index()
                    } else {
                        row.index()
                    };

                    let entry = &entries[row_index];

                    row.set_selected(self.selection.contains(&row_index));

                    row.col(|ui| {
                        ui.label(row_index.to_string());
                    });
                    row.col(|ui| {
                        ui.label(entry.timestamp.to_string());
                    });
                    row.col(|ui| {
                        let color = match entry.level {
                            tracing::Level::ERROR => egui::Color32::RED,
                            tracing::Level::WARN => egui::Color32::YELLOW,
                            tracing::Level::INFO => egui::Color32::GREEN,
                            tracing::Level::DEBUG => egui::Color32::BLUE,
                            tracing::Level::TRACE => egui::Color32::GRAY,
                        };

                        ui.label(RichText::new(entry.level.to_string()).color(color));
                    });
                    row.col(|ui| {
                        ui.label(entry.module.clone());
                    });
                    row.col(|ui| {
                        ui.label(entry.file.clone());
                    });
                    row.col(|ui| {
                        ui.label(entry.message.clone());
                    });

                    self.toggle_row_selection(row_index, &row.response());
                });
            });
    }

    fn toggle_row_selection(&mut self, row_index: usize, row_response: &egui::Response) {
        if row_response.clicked() {
            if self.selection.contains(&row_index) {
                self.selection.remove(&row_index);
            } else {
                self.selection.insert(row_index);
            }
        }
    }
}

fn expanding_content(ui: &mut egui::Ui) {
    ui.add(egui::Separator::default().horizontal());
}

fn long_text(row_index: usize) -> String {
    format!("Row {row_index} has some long text that you may want to clip, or it will take up too much horizontal space!")
}
