use std::time::{Instant, Duration};
use std::fmt::Display;
use eframe::egui;
use eframe::egui::{Widget, WidgetText};

macro_rules! rich_text {
    ($text: expr) => {
	egui::RichText::new($text)
    };
    ($text: expr $(, $data: expr)+) => {
	egui::RichText::new(format!($text, $($data),+))
    };
}
macro_rules! separated_mono {
    ($ui: ident, $text: expr) => {
	{
	    let r = $ui.label(rich_text!($text).monospace());
	    $ui.separator();
	    r
	}
    }
}
macro_rules! rect {
    ($x: expr, $y: expr, $w: expr, $h: expr) => {
	egui::Rect::from_center_size(egui::Pos2::new($x, $y), egui::Vec2::new($w, $h))
    }
}

fn main() {
    println!("Hello, world!");
    let native_options = eframe::NativeOptions{
	viewport: egui::ViewportBuilder::default()
	    .with_title("Hai Domo!")
	    //.with_resizable(false)
	    .with_min_inner_size(egui::Vec2 { x:260.0, y:480.0 })
	    .with_inner_size(egui::Vec2 { x:260.0, y:480.0 })
	//.with_always_on_top(),
	    ,
	..Default::default()
    };

    let _ = eframe::run_native("Hai Domo!", native_options, Box::new(|cc| {
	Box::new(HaiDomoApp::new(cc))
    }));
}

struct HaiDomoApp {
    started: Option<Instant>,
    timed: Duration,
}

impl HaiDomoApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
	Self {
	    started: None,
	    timed: Duration::from_secs(0),
	}
    }
}

impl eframe::App for HaiDomoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
	let timed: Timed = (self.timed + match self.started {
	    Some(x) => {
		ctx.request_repaint();
		x.elapsed()
	    },
	    None => Duration::from_secs(0)
	}).into();

	egui::TopBottomPanel::top("run_title").show(ctx, |ui| {
	    ui.heading("Ur Mom");
	});
	
	egui::CentralPanel::default().show(ctx, |ui| {
	    let max_rect = ui.max_rect();
	    ui.set_width(max_rect.width());
	    egui::ScrollArea::vertical().show(ui, |ui| {
		let p = ui.painter().clone();
		let max_rect = ui.max_rect();
		ui.set_width(max_rect.width());
		ui.vertical_centered_justified(|ui| {
		    TimedSplit::from_current("Split 1".into(), timed.clone(), None)
			.draw_ui(ui, false);
		    let background_color = egui::Color32::LIGHT_GREEN;
		    let l1 = separated_mono!(ui, "Split 1");
		    p.rect_filled(
			l1.rect,
			egui::Rounding::ZERO,
			background_color
		    );
		    p.text(
			l1.rect.center(),
			egui::Align2::CENTER_CENTER,
			"Split 1",
			egui::FontId::monospace(14.0),
			egui::Color32::BLACK
		    );
		    separated_mono!(ui, "Split 2");
		    separated_mono!(ui, "Split 3");
		});
	    });
	});
	
	egui::TopBottomPanel::bottom("current_time")
	    .frame({
		egui::Frame::none()
		    .fill(egui::Color32::LIGHT_BLUE)
		    .inner_margin(4.0)
	    }).show(ctx, |ui| {
		timed.clone().ui(ui);

		if ui.input(|i| i.key_pressed(egui::Key::Space)) {
		    match self.started {
			Some(_) => {
			    self.started = None;
			    self.timed = timed.clone_duration();
			},
			None => {
			    self.started = Some(Instant::now());
			    ctx.request_repaint();
			}
		    }
		}
	    });
    }
}

#[derive(Clone)]
struct TimedSplit {
    name: String,
    current_split: Timed,
    comparison: Option<Timed>,
}
impl TimedSplit {
    fn from_current(name: String, current_split: Timed, comparison: Option<Timed>) -> Self {
	Self {
	    name: name,
	    current_split: current_split,
	    comparison: comparison,
	}
    }
    fn draw_ui(&self, ui: &mut egui::Ui, completed: bool) -> egui::Response {
	let frame = egui::Frame::none()
	    .fill({
		if completed {
		    egui::Color32::BLACK
		} else {
		    egui::Color32::BLUE
		}
	    })
	    .inner_margin(4.0);
	let mut f = frame.begin(ui);

	let response = f.allocate_space(ui);
	egui::SidePanel::left("Split-C1").frame(frame).show_animated_inside(ui, true, |ui| {
	    ui.label(rich_text!(&self.name));
	    
	    if let Some(comp) = self.comparison.clone() {
		let a = self.current_split.clone_duration().as_secs_f64();
		let b = comp.clone_duration().as_secs_f64();
		let diff = b - a;
		let text = if diff > 1.0 {
		    let dt:Timed = Duration::from_secs_f64(diff).into();
		    if dt.hours > 0 {
			rich_text!("+{}:{:02}:{:02}.{:03}", dt.hours, dt.minutes, dt.seconds, dt.milliseconds)
		    } else if dt.minutes > 0 {
			rich_text!("+{}:{:02}.{:03}", dt.minutes, dt.seconds, dt.milliseconds)
		    } else {
			rich_text!("+{}.{:03}", dt.seconds, dt.milliseconds)
		    }.color(egui::Color32::RED)
		} else if diff < 1.0 {
		    let dt:Timed = Duration::from_secs_f64(-1.0*diff).into();
		    if dt.hours > 0 {
			rich_text!("-{}:{:02}:{:02}.{:03}", dt.hours, dt.minutes, dt.seconds, dt.milliseconds)
		    } else if dt.minutes > 0 {
			rich_text!("-{}:{:02}.{:03}", dt.minutes, dt.seconds, dt.milliseconds)
		    } else {
			rich_text!("-{}.{:03}", dt.seconds, dt.milliseconds)
		    }.color(egui::Color32::GOLD)
		} else if diff < 0.0 {
		    let dt:Timed = Duration::from_secs_f64(-1.0*diff).into();
		    if dt.hours > 0 {
			rich_text!("-{}:{:02}:{:02}.{:03}", dt.hours, dt.minutes, dt.seconds, dt.milliseconds)
		    } else if dt.minutes > 0 {
			rich_text!("-{}:{:02}.{:03}", dt.minutes, dt.seconds, dt.milliseconds)
		    } else {
			rich_text!("-{}.{:03}", dt.seconds, dt.milliseconds)
		    }.color(egui::Color32::GREEN)
		} else {
		    let dt:Timed = Duration::from_secs_f64(diff).into();
		    if dt.hours > 0 {
			rich_text!("+{}:{:02}:{:02}.{:03}", dt.hours, dt.minutes, dt.seconds, dt.milliseconds)
		    } else if dt.minutes > 0 {
			rich_text!("+{}:{:02}.{:03}", dt.minutes, dt.seconds, dt.milliseconds)
		    } else {
			rich_text!("+{}.{:03}", dt.seconds, dt.milliseconds)
		    }.color(egui::Color32::YELLOW).size(16.0)
		};

		f.content_ui.label(text);
	    }
	});

	{
	    let dt = &self.current_split;
	    if dt.hours > 0 {
		rich_text!("+{}:{:02}:{:02}.{:03}", dt.hours, dt.minutes, dt.seconds, dt.milliseconds)
	    } else if dt.minutes > 0 {
		rich_text!("+{}:{:02}.{:03}", dt.minutes, dt.seconds, dt.milliseconds)
	    } else {
		rich_text!("+{}.{:03}", dt.seconds, dt.milliseconds)
	    }.color(egui::Color32::RED).size(16.0);
	}
	
	f.paint(ui);
	f.end(ui);

	return response;
    }
}

#[derive(Clone)]
struct Timed {
    total_seconds: u64,
    subsec_nanos: u32,
    hours: u64,
    minutes: u64,
    seconds: u64,
    milliseconds: u32,
}
impl Timed {
    fn clone_duration(&self) -> Duration {
	self.clone().into()
    }
    fn draw_ui(&self, ui: &mut egui::Ui, main_size: f32, millis_size: f32) -> egui::Response {
	let hours_minutes_seconds = if self.hours > 0 {
	    egui::RichText::new(format!("{:02}:{:02}:{:02}", self.hours, self.minutes, self.seconds))
	} else {
	    egui::RichText::new(format!("{:02}:{:02}", self.minutes, self.seconds))
	}.monospace().color(egui::Color32::BLACK).line_height(Some(main_size - 2.0)).size(main_size);
	
	let milliseconds = egui::RichText::new(format!(".{:03}", self.milliseconds)).monospace().size(millis_size);
	let inner_response = ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
	    let style = egui::Style::default();
	    let mut job = egui::text::LayoutJob::default();
	    hours_minutes_seconds.append_to(
		&mut job,
		&style,
		egui::FontSelection::Default,
		egui::Align::BOTTOM
	    );
	    milliseconds.append_to(
		&mut job,
		&style,
		egui::FontSelection::Default,
		egui::Align::BOTTOM
	    );
	    
	    ui.label(job)
	});

	return inner_response.inner;
    }
}
impl Widget for Timed {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
	self.draw_ui(ui, 64.0, 24.0)
    }
}
impl Display for Timed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
	if self.hours > 0 {
	    write!(f, "{:02}:{:02}:{:02}:{:03}", self.hours, self.minutes, self.seconds, self.milliseconds)
	} else {
	    write!(f, "{:02}:{:02}:{:03}", self.minutes, self.seconds, self.milliseconds)
	}
    }
}
impl Into<String> for Timed {
    fn into(self) -> String {
	format!("{:02}:{:02}:{:02}:{:03}", self.hours, self.minutes, self.seconds, self.milliseconds)
    }
}
impl From<Duration> for Timed {
    fn from(time: Duration) -> Self {
	let total_secs = time.as_secs();
	let mins = total_secs / 60;
	let hours = mins / 60;
	let mins = mins % 60;
	let secs = total_secs % 60;
	let mili = time.subsec_millis();
	Self {
	    total_seconds: total_secs,
	    subsec_nanos: time.subsec_nanos(),
	    hours: hours,
	    minutes: mins,
	    seconds: secs,
	    milliseconds: mili,
	}
    }
}
impl Into<Duration> for Timed {
    fn into(self) -> Duration {
	Duration::new(self.total_seconds, self.subsec_nanos)
    }
}
