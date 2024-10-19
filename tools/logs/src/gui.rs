#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::{
    entries::LogEntry,
    table::{LogsTable, View},
};
use eframe::{egui, NativeOptions};
use egui::Context;
use std::{fs, path::PathBuf};

pub fn open_gui() -> eframe::Result {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 800.0]),
        centered: true,
        ..Default::default()
    };

    let mut app = Application::new();

    eframe::run_simple_native("Roan's Logs Viewer", options, move |ctx, _frame| {
        app.run(ctx).expect("Failed to run app");
    })
}

#[derive(Default)]
struct Application {
    picked_path: Option<String>,
    log_entries: Vec<LogEntry>, // Store parsed log entries
}

impl Application {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self, ctx: &Context) -> eframe::Result {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Open file…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.picked_path = Some(path.display().to_string());

                    if let Some(picked_path) = &self.picked_path {
                        let content = fs::read_to_string(PathBuf::from(picked_path)).unwrap();
                        self.log_entries.clear(); // Clear previous entries
                        for line in content.lines() {
                            let entry = LogEntry::from_string(line);
                            self.log_entries.push(entry);
                        }
                    }
                }
            }

            if let Some(picked_path) = &self.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path);
                });

                for entry in &self.log_entries {
                    ui.label(format!("{:?}", entry));
                }
            }

            LogsTable::default().ui(ui);
        });

        Ok(())
    }
}
