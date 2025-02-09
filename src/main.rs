mod stopwatch;
use stopwatch::*;

mod splits_file;
use splits_file::RunData;

use eframe::egui;
use eframe::egui::Widget;
use std::fmt::Display;
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
            let run_data = RunData::new(String::from("UrMom"), split_names);
            Box::new(HaiDomoApp::new_with_splits(cc, sw, run_data))
        }),
    )
}

struct HaiDomoApp {
    stopwatch: Stopwatch,
    splits: Vec<(usize, StopSplit)>,
    run_data: RunData,
    at: usize,
}

impl HaiDomoApp {
    fn new(_cc: &eframe::CreationContext<'_>, run_name: String) -> Self {
        println!("[INFO] Creating HaiDomoApp...");
        Self {
            stopwatch: Stopwatch::new(),
            splits: Vec::new(),
            run_data: RunData::new(run_name, vec![]),
            at: 0,
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
        Self {
            stopwatch: stopwatch,
            splits: splits,
            run_data: run_data,
            at: 0,
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
}

impl eframe::App for HaiDomoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.stopwatch.is_running() {
            ctx.request_repaint();
        }
        let timestamp = self.timestamp().expanded();

        egui::TopBottomPanel::top("run_title").show(ctx, |ui| {
            ui.heading("Ur Mom");
            let inner_response = ui.horizontal(|ui| ui.label("Any%"));
            let sense = egui::Sense::click().union(egui::Sense::hover());
            let innr = inner_response.inner;
            let resp = inner_response.response;
            let resp = ui.interact(resp.rect, resp.id, sense.clone());
            let bgrs = ui.interact_bg(sense.clone());
            if resp.double_clicked() || innr.double_clicked() {
                println!("[INFO] Double clicked area");
            } else if bgrs.double_clicked() {
                if let Some(mouse) = ui.input(|i| i.pointer.interact_pos()) {
                    if resp.rect.top() < mouse.y && mouse.y < resp.rect.bottom() {
                        println!("[INFO] Double clicked lower background");
                    } else {
                        println!("[INFO] Double clicked upper background");
                    }
                }
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
                        let data = &s.1;
                        ui.horizontal(|ui| {
                            // Display: $name | split-data
                            ui.label(rich_text!(name).monospace());
                            ui.separator();
                            data.show(ui, &self.stopwatch);
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
                timestamp.show(ui, 64.0, 32.0);

                if ui.input(|i| i.key_pressed(egui::Key::Space)) {
                    if !self.is_started() {
                        self.start_timer();
                    } else if self.stopwatch.toggle() {
                        println!("[INFO] Stopwatch has been turned on");
                        ctx.request_repaint();
                    } else {
                        println!("[INFO] Stopwatch has been turned off");
                    }
                } else if ui.input(|i| i.key_pressed(egui::Key::S)) {
                    self.next_split();
                }
            });
    }
}
