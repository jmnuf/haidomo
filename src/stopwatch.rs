use eframe::egui;
use std::time::{Duration, Instant};

#[inline]
fn zero_dur() -> Duration {
    Duration::from_nanos(0)
}
pub struct Timestamp {
    seconds: u64,
    subsecs: u32,
}
impl Timestamp {
    pub fn duration(&self) -> Duration {
        Duration::new(self.seconds, self.subsecs)
    }

    pub fn seconds(&self) -> f64 {
        self.duration().as_secs_f64()
    }

    pub fn expanded(&self) -> ExpandedTimestamp {
        ExpandedTimestamp::from(self.duration())
    }
}
impl Into<Duration> for Timestamp {
    fn into(self) -> Duration {
        Duration::new(self.seconds, self.subsecs)
    }
}
impl From<Duration> for Timestamp {
    fn from(duration: Duration) -> Self {
        Self {
            seconds: duration.as_secs(),
            subsecs: duration.subsec_nanos(),
        }
    }
}

pub struct ExpandedTimestamp {
    pub hours: u64,
    pub minutes: u64,
    pub seconds: u64,
    pub milliseconds: u32,
}
impl ExpandedTimestamp {
    pub fn simple_text(&self) -> String {
        if self.hours > 0 {
            format!("{:02}:{:02}:{:02}", self.hours, self.minutes, self.seconds)
        } else {
            format!("{:02}:{:02}", self.minutes, self.seconds)
        }
    }
    pub fn millis_text(&self) -> String {
        format!("{:03}", self.milliseconds)
    }

    pub fn show(&self, ui: &mut egui::Ui, main_size: f32, millis_size: f32) -> egui::Response {
        let hours_minutes_seconds = if self.hours > 0 {
            egui::RichText::new(format!(
                "{:02}:{:02}:{:02}",
                self.hours, self.minutes, self.seconds
            ))
        } else {
            egui::RichText::new(format!("{:02}:{:02}", self.minutes, self.seconds))
        }
        .monospace()
        .color(egui::Color32::BLACK)
        .line_height(Some(main_size - 2.0))
        .size(main_size);

        let milliseconds = egui::RichText::new(format!(".{:03}", self.milliseconds))
            .monospace()
            .size(millis_size);
        let inner_response = ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                let style = egui::Style::default();
                let mut job = egui::text::LayoutJob::default();
                hours_minutes_seconds.append_to(
                    &mut job,
                    &style,
                    egui::FontSelection::Default,
                    egui::Align::BOTTOM,
                );
                milliseconds.append_to(
                    &mut job,
                    &style,
                    egui::FontSelection::Default,
                    egui::Align::BOTTOM,
                );

                ui.label(job)
            },
        );

        return inner_response.inner;
    }
}

impl std::fmt::Display for ExpandedTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{:02}:{:02}:{:02}.{:03}",
            self.hours, self.minutes, self.seconds, self.milliseconds
        )
    }
}
impl From<Duration> for ExpandedTimestamp {
    fn from(time: Duration) -> Self {
        let total_secs = time.as_secs();
        let millis = time.subsec_millis();
        let mins = total_secs / 60;
        let hours = mins / 60;
        let mins = mins % 60;
        let secs = total_secs % 60;
        Self {
            hours: hours,
            minutes: mins,
            seconds: secs,
            milliseconds: millis,
        }
    }
}

pub struct Stopwatch {
    start_time: Option<Instant>,
    elapsed: Duration,
}

impl Stopwatch {
    pub fn start_new() -> Self {
        Self {
            start_time: Some(Instant::now()),
            elapsed: zero_dur(),
        }
    }

    pub fn new() -> Self {
        Self {
            start_time: None,
            elapsed: zero_dur(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.start_time.is_some()
    }

    pub fn time_elapsed(&self) -> Duration {
        self.elapsed.clone()
            + match self.start_time {
                None => Duration::ZERO,
                Some(x) => x.elapsed(),
            }
    }

    pub fn timestamp(&self) -> Timestamp {
        Timestamp::from(self.time_elapsed())
    }

    pub fn start(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }
    }

    pub fn pause(&mut self) -> Duration {
        if let Some(_) = self.start_time {
            let total_elapsed = self.time_elapsed();
            self.start_time = None;
            self.elapsed = total_elapsed;
            total_elapsed.clone()
        } else {
            self.elapsed.clone()
        }
    }

    pub fn toggle(&mut self) -> bool {
        if self.is_running() {
            let _ = self.pause();
            false
        } else {
            self.start();
            true
        }
    }

    pub fn clear(&mut self) {
        self.start_time = None;
        self.elapsed = zero_dur();
    }

    pub fn update_start_time(&mut self) -> Duration {
        let split_time = self.time_elapsed();
        if let Some(_) = self.start_time {
            let total_elapsed = self.time_elapsed();
            self.start_time = Some(Instant::now());
            self.elapsed = total_elapsed;
        }
        return split_time;
    }
}
impl egui::Widget for Stopwatch {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let elapsed = self.time_elapsed();
        let timestamp = Timestamp::from(elapsed).expanded();
        timestamp.show(ui, 64.0, 18.0)
    }
}

pub struct StopSplit {
    split_start: Option<Duration>,
    elapsed: Duration,
    completed: bool,
}
impl StopSplit {
    pub fn new() -> Self {
        Self {
            split_start: None,
            elapsed: zero_dur(),
            completed: false,
        }
    }

    pub fn new_started(sw: &Stopwatch) -> Self {
        let split_start = sw.time_elapsed();
        Self {
            split_start: Some(split_start),
            elapsed: zero_dur(),
            completed: false,
        }
    }

    pub fn not_started(&self) -> bool {
        self.split_start.is_none()
    }

    pub fn is_done(&self) -> bool {
        !self.not_started() && self.completed
    }

    pub fn time_elapsed(&self, sw: &Stopwatch) -> Duration {
        // Not started split just returns zero
        if self.not_started() {
            return zero_dur();
        }
        if self.completed {
            return self.elapsed;
        }
        let elapsed = sw.time_elapsed() - self.split_start.unwrap();
        return elapsed;
    }

    pub fn start(&mut self, sw: &Stopwatch) {
        if !self.not_started() {
            return;
        }
        if self.is_done() {
            return;
        }
        self.elapsed = zero_dur();
        self.split_start = Some(sw.time_elapsed());
    }

    pub fn start_at_zero(&mut self) {
        if !self.not_started() {
            return;
        }
        if self.is_done() {
            return;
        }
        self.elapsed = zero_dur();
        self.split_start = Some(zero_dur());
    }

    pub fn stop(&mut self, sw: &Stopwatch) {
        if !self.not_started() {
            let elapsed = sw.time_elapsed() - self.split_start.unwrap();
            self.elapsed = elapsed;
            self.completed = true;
        }
    }

    pub fn resume(&mut self) {
        if !self.not_started() {
            self.completed = false;
        }
    }

    pub fn toggle_split(&mut self, sw: &Stopwatch) {
        if self.not_started() {
            self.start(sw);
        } else if self.completed {
            self.resume();
        } else {
            self.stop(sw);
        }
    }

    pub fn clear(&mut self) {
        self.split_start = None;
        self.elapsed = zero_dur();
        self.completed = false;
    }

    pub fn show(&self, ui: &mut egui::Ui, sw: &Stopwatch) {
        let elapsed: ExpandedTimestamp = self.time_elapsed(sw).into();
        elapsed.show(ui, 16.0, 10.0);
    }
}
