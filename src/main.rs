use rfd;

mod stopwatch;
use stopwatch::*;

mod splits_file;
use splits_file::{RunData, RunDataFileError};

use eframe::egui;
use eframe::egui::Widget;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::time::Duration;

macro_rules! rich_text {
    ($text: expr) => {
	egui::RichText::new($text)
    };
    ($text: expr $(, $data: expr)+) => {
	egui::RichText::new(format!($text, $($data),+))
    };
}

fn main() -> Result<(), eframe::Error> {
    let width = 280.0;
    let height = 480.0;
    println!("[INFO] Starting with window size: {width}x{height}");
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Hai Domo!")
            //.with_resizable(false)
            .with_min_inner_size(egui::Vec2 {
                x: width,
                y: height,
            })
            .with_inner_size(egui::Vec2 {
                x: width,
                y: height,
            }), //.with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        "Hai Domo!",
        native_options,
        Box::new(|cc| {
            let sw = Stopwatch::new();
            let mut splits = Vec::new();
            let mut split_names = Vec::new();
            for i in 1..4 {
                let name = format!("Split-{:02}", i);
                let data = StopSplit::new();
                let split = (i, data);
                splits.push(split);
                split_names.push(name);
            }
            let run_data = RunData::new(String::from("UrMom:Any%"), split_names);
            Box::new(HaiDomoApp::new_with_splits(cc, sw, run_data))
        }),
    )
}

struct HaiDomoApp {
    stopwatch: Stopwatch,
    splits: Vec<(usize, StopSplit)>,
    run_data: RunData,
    at: usize,

    // State
    run_title: String,
    run_subtitle: String,
    editing_run_title: bool,
    editing_run_subtitle: bool,
}

impl HaiDomoApp {
    fn new(_cc: &eframe::CreationContext<'_>, run_title: String, run_subtitle: String) -> Self {
        println!("[INFO] Creating HaiDomoApp...");
        Self {
            stopwatch: Stopwatch::new(),
            splits: Vec::new(),
            run_data: RunData::new(format!("{run_title}:{run_subtitle}"), vec![]),
            at: 0,

            run_title: run_title,
            run_subtitle: run_subtitle,
            editing_run_title: false,
            editing_run_subtitle: false,
        }
    }

    fn new_with_splits(
        _cc: &eframe::CreationContext<'_>,
        stopwatch: Stopwatch,
        run_data: RunData,
    ) -> Self {
        let splits: Vec<_> = run_data
            .get_indexed_split_names()
            .iter()
            .map(|(idx, _)| (*idx, StopSplit::new()))
            .collect();
        println!("[INFO] Creating HaiDomoApp with {} splits...", splits.len());
        let title = run_data.get_title().to_string();
        let subtitle = run_data.get_subtitle().unwrap_or("").to_string();
        Self {
            stopwatch: stopwatch,
            splits: splits,
            run_data: run_data,
            at: 0,

            run_title: title,
            run_subtitle: subtitle,
            editing_run_title: false,
            editing_run_subtitle: false,
        }
    }

    fn add_split(&mut self, name: String) {
        match self.run_data.add_split(name) {
            Ok(i) => {
                let data = StopSplit::new();
                let split = (i, data);
                self.splits.push(split);
            }
            Err(_) => {
                eprintln!("[ERROR] Failed to add new split! Max splits reached already?");
            }
        };
    }

    fn get_split_name(&self, idx: usize) -> Option<&String> {
        self.run_data.get_split_name(idx)
    }

    fn timestamp(&self) -> Timestamp {
        self.stopwatch.timestamp()
    }

    fn is_started(&self) -> bool {
        self.stopwatch.is_running() || !self.stopwatch.time_elapsed().is_zero()
    }

    fn is_timer_running(&self) -> bool {
        self.stopwatch.is_running()
    }

    fn start_timer(&mut self) {
        self.stopwatch.clear();
        for s in self.splits.iter_mut() {
            let split = &mut s.1;
            split.clear();
        }
        self.stopwatch.start();
        if self.splits.len() >= 1 {
            let s = &mut self.splits[0];
            let split = &mut s.1;
            split.start_at_zero();
        }
    }

    fn stop_timer(&mut self) {
        self.stopwatch.pause();
        if self.splits.is_empty() {
            return;
        }
        for s in self.splits.iter_mut() {
            let split = &mut s.1;
            if !split.is_done() {
                split.stop(&self.stopwatch);
            }
        }
    }

    fn next_split(&mut self) {
        self.at += 1;
        if self.at >= self.splits.len() {
            self.stop_timer();
            return;
        }

        let prev = &mut self.splits.get_mut(self.at - 1).unwrap().1;
        prev.stop(&self.stopwatch);
        let next = &mut self.splits.get_mut(self.at).unwrap().1;
        next.start(&self.stopwatch);
    }

    fn save_run_data(&self, f: &mut File) -> Result<(), (&str, String)> {
        match self.run_data.write_to(f) {
            Err(e) => match e {
                RunDataFileError::IOError(err) => {
                    let msg = (
                        "Failed to save run data",
                        format!("Issue occured when writing file contents:\n  {err}"),
                    );
                    Err(msg)
                }
                RunDataFileError::ByteGenError(msg) => {
                    let msg = (
                        "Failed to save run data",
                        format!("Issue occured when writing file contents:\n  {msg}"),
                    );
                    Err(msg)
                }
                RunDataFileError::ParseError(_) => {
                    unreachable!("Should never get a parse error when saving a file")
                }
            },
            Ok(_) => {
                if let Err(err) = f.flush() {
                    let msg = (
                        "Failed to save run data",
                        format!("Issue occured when writing file contents:\n  {err}"),
                    );
                    Err(msg)
                } else {
                    Ok(())
                }
            }
        }
    }
}

impl eframe::App for HaiDomoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.stopwatch.is_running() {
            ctx.request_repaint();
        }
        let timestamp = self.timestamp().expanded();

        egui::TopBottomPanel::top("run_name").show(ctx, |ui| {
            if self.editing_run_title {
                let resp = ui.text_edit_singleline(&mut self.run_title);
                self.run_data.set_title(self.run_title.as_str());
                if resp.lost_focus() {
                    self.editing_run_title = false;
                } else {
                    resp.request_focus();
                }
            } else {
                let resp = ui.heading(&self.run_title);
                if resp.double_clicked() {
                    self.editing_run_title = true;
                    self.editing_run_subtitle = false;
                }
            }
            let resp = if self.editing_run_subtitle {
                let resp = ui.text_edit_singleline(&mut self.run_subtitle);
                self.run_data.set_subtitle(self.run_subtitle.as_str());
                if resp.lost_focus() {
                    self.editing_run_subtitle = false;
                } else {
                    resp.request_focus();
                }
                resp
            } else {
                let resp = ui.label(&self.run_subtitle);
                if resp.double_clicked() {
                    self.editing_run_subtitle = true;
                    self.editing_run_title = false;
                }
                resp
            };
            // Only be able to edit title/subtitle when stopwatch is not running
            if !self.stopwatch.is_running() {
                let sense = egui::Sense::click().union(egui::Sense::hover());
                let bg_resp = ui.interact_bg(sense);
                if bg_resp.double_clicked() {
                    if let Some(mouse) = ui.input(|i| i.pointer.interact_pos()) {
                        if resp.rect.top() < mouse.y && mouse.y < resp.rect.bottom() {
                            self.editing_run_subtitle = !self.editing_run_subtitle;
                            self.editing_run_title = false;
                        } else {
                            self.editing_run_title = !self.editing_run_title;
                            self.editing_run_subtitle = false;
                        }
                    }
                }
            } else {
                self.editing_run_title = false;
                self.editing_run_subtitle = false;
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let max_rect = ui.max_rect();
            ui.set_width(max_rect.width());
            egui::ScrollArea::vertical().show(ui, |ui| {
                let max_rect = ui.max_rect();
                ui.set_width(max_rect.width());
                ui.vertical_centered_justified(|ui| {
                    for s in self.splits.iter() {
                        let name = self.get_split_name(*&s.0).unwrap();
                        let split = &s.1;
                        ui.horizontal(|ui| {
                            // Display: $name | split-data
                            ui.label(rich_text!(name).monospace());
                            ui.separator();
                            split.show(ui, &self.stopwatch);
                        });
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("current_time")
            .frame({
                egui::Frame::none()
                    .fill(egui::Color32::LIGHT_BLUE)
                    .inner_margin(4.0)
            })
            .show(ctx, |ui| {
		use egui::{Modifiers, Key};

                timestamp.show(ui, 64.0, 32.0);

                if !self.editing_run_title && !self.editing_run_subtitle {
                    if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Space)) {
                        if !self.is_started() {
                            self.start_timer();
                        } else if self.stopwatch.toggle() {
                            println!("[INFO] Stopwatch has been turned on");
                            ctx.request_repaint();
                        } else {
                            println!("[INFO] Stopwatch has been turned off");
                        }
                    } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::S)) {
                        self.next_split();
                    }
		    if !self.stopwatch.is_running() {
			if ui.input_mut(|i| i.consume_key(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::S)) {
			    let file_path = rfd::FileDialog::new()
				.add_filter("Binary Six Shooter", &["binss"])
				.set_file_name(self.run_data.get_name())
				.set_title("Save run data")
				.save_file();
			    if let Some(file_path) = file_path {
				match File::create(file_path) {
				    Err(e) => {
					rfd::MessageDialog::new()
					    .set_title("Failed to save run data")
					    .set_level(rfd::MessageLevel::Error)
					    .set_description(format!("Issue occured when opening file: {e}"))
					    .set_buttons(rfd::MessageButtons::Ok)
					    .show();
				    },
				    Ok(mut f) => {
					match self.save_run_data(&mut f) {
					    Err((title, message)) => {
						rfd::MessageDialog::new()
						    .set_title(title)
						    .set_level(rfd::MessageLevel::Error)
						    .set_description(message)
						    .set_buttons(rfd::MessageButtons::Ok)
						    .show();
					    },
					    Ok(_) => {
						rfd::MessageDialog::new()
						    .set_title("Saved Run Data")
						    .set_level(rfd::MessageLevel::Info)
						    .set_description("Succesfully saved your binary six shooter.\nShe's a sweet six shooter, she knows how to get down!")
						    .set_buttons(rfd::MessageButtons::Ok)
						    .show();
					    }
					};
				    },
				};
			    }
			}
		    }
                }
            });
    }
}
