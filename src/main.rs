mod stopwatch;
use stopwatch::*;

mod splits_file;

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
            for i in 1..4 {
                let name = format!("Split-{:02}", i);
                let data = StopSplit::new();
                let split = (name, data);
                splits.push(split);
            }
            Box::new(HaiDomoApp::new_with_splits(cc, sw, splits))
        }),
    )
}

struct HaiDomoApp {
    stopwatch: Stopwatch,
    splits: Vec<(String, StopSplit)>,
    at: usize,
}

impl HaiDomoApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        println!("[INFO] Creating HaiDomoApp...");
        Self {
            stopwatch: Stopwatch::new(),
            splits: Vec::new(),
            at: 0,
        }
    }

    fn new_with_splits(
        _cc: &eframe::CreationContext<'_>,
        stopwatch: Stopwatch,
        splits: Vec<(String, StopSplit)>,
    ) -> Self {
        println!("[INFO] Creating HaiDomoApp with {} splits...", splits.len());
        Self {
            stopwatch: stopwatch,
            splits: splits,
            at: 0,
        }
    }

    fn add_split(&mut self, name: String) {
        let data = StopSplit::new();
        let split = (name, data);
        self.splits.push(split);
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
            ui.label("Any%");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let max_rect = ui.max_rect();
            ui.set_width(max_rect.width());
            egui::ScrollArea::vertical().show(ui, |ui| {
                let max_rect = ui.max_rect();
                ui.set_width(max_rect.width());
                ui.vertical_centered_justified(|ui| {
                    for s in self.splits.iter() {
                        let name = &s.0;
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
